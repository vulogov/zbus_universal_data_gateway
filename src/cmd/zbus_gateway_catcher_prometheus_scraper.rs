extern crate log;
use crate::cmd;
use crate::stdlib;
use reqwest;

pub fn catcher(_c: &cmd::Cli, gateway: &cmd::Gateway)  {
    log::trace!("zbus_gateway_catcher_prometheus_scraper::run() reached");
    let gateway = gateway.clone();
    match stdlib::threads::THREADS.lock() {
        Ok(t) => {
            t.execute(move ||
            {
                log::debug!("Prometheus scraper catcher thread has been started");
                loop {
                    log::debug!("Running Prometheus scrape");
                    for e in &gateway.prometheus_exporter_connect {
                        log::debug!("Attempting to scrape: {}", &e);
                        match reqwest::blocking::get(e.clone()) {
                            Ok(body) => {
                                let data: String = match body.text() {
                                    Ok(data) => String::from(data),
                                    Err(err) => {
                                        log::error!("Exporter returned an empty response: {}", err);
                                        continue;
                                    }
                                };
                                log::debug!("Scraped len()={} bytes from {}", &data.len(), &e);
                                stdlib::channel::pipe_push("in".to_string(), data);
                            }
                            Err(err) => {
                                log::error!("Prometheus scraping error: {}", err);
                            }
                        }

                    }
                    stdlib::sleep::sleep(gateway.prometheus_scraper_run_every.into());
                }
            });
            drop(t);
        }
        Err(err) => {
            log::error!("Error accessing Thread Manager: {:?}", err);
            return;
        }
    }
}
