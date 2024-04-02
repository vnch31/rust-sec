use futures::stream;
use futures::StreamExt;
use reqwest::Client;
use std::collections::HashSet;
use std::time::Duration;
use std::time::Instant;

use crate::dns;
use crate::modules::HttpModule;
// use crate::modules::HttpModule;
use crate::modules::Subdomain;
use crate::ports;
use crate::{modules, Error};

pub fn modules() {
    let http_modules = modules::http_modules();
    let subdomains_modules = modules::subdomain_modules();

    println!("Subdomains modules");
    for module in subdomains_modules {
        println!("   {}: {}", module.name(), module.description());
    }

    println!("HTTP modules");
    for module in http_modules {
        println!("    {}: {}", module.name(), module.description());
    }
}

pub fn scan(target: &str) -> Result<(), Error> {
    log::info!("Scanning target: {}", target);

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Building tokio's runtime");

    let http_timeout = Duration::from_secs(30);
    let http_client = Client::builder().timeout(http_timeout).build()?;
    let dns_resolver = dns::new_resolver();

    let subdomains_concurrency = 20;
    let dns_concurrency = 100;
    let ports_concurrency = 200;
    let vulnerabilities_conccurency = 20;
    let scan_start = Instant::now();

    let subdomains_modules = modules::subdomain_modules();

    runtime.block_on(async move {
        // scan subdomains
        let mut subdomains: Vec<String> = stream::iter(subdomains_modules.into_iter())
            .map(|module| async move {
                match module.enumerate(target).await {
                    Ok(new_subdomains) => Some(new_subdomains),
                    Err(err) => {
                        log::error!("subdomains/{}: {}", module.name(), err);
                        None
                    }
                }
            })
            .buffer_unordered(subdomains_concurrency)
            .filter_map(|domain| async { domain })
            .collect::<Vec<Vec<String>>>()
            .await
            .into_iter()
            .flatten()
            .collect();

        subdomains.push(target.to_string());

        // dedup, clean and convert results
        let subdomains: Vec<Subdomain> = HashSet::<String>::from_iter(subdomains.into_iter())
            .into_iter()
            .filter(|subdomain| subdomain.contains(target))
            .map(|domain| Subdomain {
                domain,
                open_ports: vec![],
            })
            .collect();

        log::info!("Found {} subdomains", subdomains.len());

        // filter unresolvable domains
        let subdomains: Vec<Subdomain> = stream::iter(subdomains.into_iter())
            .map(|domain| dns::resolves(&dns_resolver, domain))
            .buffer_unordered(dns_concurrency)
            .filter_map(|domain| async move { domain })
            .collect()
            .await;

        // scan ports
        let subdomains: Vec<Subdomain> = stream::iter(subdomains.into_iter())
            .map(|domain| {
                log::info!("Scanning ports for: {}", domain.domain);
                ports::scan_ports(ports_concurrency, domain)
            })
            .buffer_unordered(1)
            .collect()
            .await;

        for subdomain in subdomains.clone() {
            log::info!(
                "{}: {:?}",
                subdomain.domain,
                subdomain
                    .open_ports
                    .into_iter()
                    .map(|port| port.port)
                    .collect::<Vec<u16>>()
            );
        }

        // scan vulnerabilities
        let mut targets: Vec<(Box<dyn HttpModule>, String)> = Vec::new();
        for subdomain in subdomains {
            for port in subdomain.open_ports {
                let http_modules = modules::http_modules();
                for http_module in http_modules {
                    let target = format!("http://{}:{}", subdomain.domain, port.port);
                    targets.push((http_module, target));
                }
            }
        }

        stream::iter(targets.into_iter())
            .for_each_concurrent(vulnerabilities_conccurency, |(module, target)| {
                let http_client = http_client.clone();

                async move {
                    match module.scan(&http_client, &target).await {
                        Ok(Some(finding)) => println!("{:?}", &finding),
                        Ok(None) => {}
                        Err(err) => log::debug!("{}: {}", target, err),
                    }
                }
            })
            .await;
    });

    let scan_duration = scan_start.elapsed();
    log::info!("Scan completed in {:?}", scan_duration);

    Ok(())
}
