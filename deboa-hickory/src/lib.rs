use deboa::{
    dns::{DnsResolver, DnsResponse},
    errors::{DeboaError, DnsError},
    Result,
};
use hickory_resolver::{
    net::runtime::RuntimeProvider,
    proto::rr::{
        rdata::{svcb::SvcParamKey, HTTPS},
        RecordData, RecordType,
    },
    Resolver,
};
use http::Version;
use rand::seq::SliceRandom;
use std::{net::IpAddr, str::FromStr, sync::Arc};

pub struct HickoryDnsResolver<P>
where
    P: RuntimeProvider,
{
    resolver: Arc<Resolver<P>>,
}

impl<P> HickoryDnsResolver<P>
where
    P: RuntimeProvider,
{
    pub fn new(resolver: Arc<Resolver<P>>) -> Self {
        Self { resolver }
    }
}

impl<P> Default for HickoryDnsResolver<P>
where
    P: RuntimeProvider + Default,
{
    fn default() -> Self {
        Self {
            resolver: Arc::new(
                Resolver::builder(P::default())
                    .expect("Could not create builder!")
                    .build()
                    .expect("Coult not build resolver!"),
            ),
        }
    }
}

impl<P> DnsResolver for HickoryDnsResolver<P>
where
    P: RuntimeProvider,
{
    async fn resolve(&self, host: String, _port: u16) -> Result<DnsResponse> {
        let resolver = self
            .resolver
            .clone();

        let lookup = resolver
            .lookup(&host, RecordType::HTTPS)
            .await
            .map_err(|e| {
                DeboaError::Dns(DnsError::Resolve { host: host.clone(), message: e.to_string() })
            })?;

        let dns_response = if let Some(rec) = lookup
            .answers()
            .iter()
            .next()
        {
            if let Some(svcb) = HTTPS::try_borrow(&rec.data) {
                let mut dns_response = DnsResponse::builder();
                let mut ips = Vec::new();
                for (key, value) in &svcb.svc_params {
                    match key {
                        SvcParamKey::Alpn => {
                            dns_response = dns_response.protos(
                                value
                                    .to_string()
                                    .split(',')
                                    .map(|alpn| match alpn {
                                        "http/1.1" => Version::HTTP_11,
                                        "h2" => Version::HTTP_2,
                                        "h3" => Version::HTTP_3,
                                        _ => Version::HTTP_11,
                                    })
                                    .collect::<Vec<Version>>(),
                            );
                        }
                        SvcParamKey::Ipv4Hint | SvcParamKey::Ipv6Hint => {
                            for addr in value
                                .to_string()
                                .split(',')
                            {
                                if !addr.is_empty() {
                                    ips.push(IpAddr::from_str(addr).map_err(|e| {
                                        DeboaError::Dns(DnsError::Resolve {
                                            host: host.clone(),
                                            message: e.to_string(),
                                        })
                                    })?);
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Some(dns_response.addresses(ips))
            } else {
                None
            }
        } else {
            None
        };

        let ips_with_alpn = if let Some(data) = dns_response {
            data.build()
        } else {
            let addrs = resolver
                .lookup_ip(&host)
                .await
                .map_err(|e| {
                    DeboaError::Dns(DnsError::Resolve {
                        host: host.clone(),
                        message: e.to_string(),
                    })
                })?;

            let mut ips: Vec<IpAddr> = addrs
                .iter()
                .collect();

            ips.shuffle(&mut rand::rng());

            DnsResponse::builder()
                .addresses(ips)
                .build()
        };

        // FIXME: Remind DnsResponse::protos() is not actually being used anywhere

        Ok(ips_with_alpn)
    }
}
