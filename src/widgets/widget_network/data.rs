use crate::Result;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Interface {
    pub name: String,
    pub address: String,
    pub loopback: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Network {
    pub interfaces: Vec<Interface>,
    pub selected: Option<Interface>,
}

pub fn read(selected: &str) -> Result<Network> {
    let mut interfaces: Vec<_> = if_addrs::get_if_addrs()
        .map_err(|error| error.to_string())?
        .into_iter()
        .map(|interface| Interface {
            name: interface.name.clone(),
            address: interface.ip().to_string(),
            loopback: interface.is_loopback(),
        })
        .collect();
    interfaces.sort_by(|a, b| a.name.cmp(&b.name).then(a.address.cmp(&b.address)));
    let default = std::fs::read_to_string("/proc/net/route")
        .unwrap_or_default()
        .lines()
        .skip(1)
        .find_map(|line| {
            let fields: Vec<_> = line.split_whitespace().collect();
            if fields.get(1) == Some(&"00000000") {
                fields.first().map(|name| name.to_string())
            } else {
                None
            }
        });
    let chosen = if selected.is_empty() {
        interfaces
            .iter()
            .filter(|interface| !interface.loopback)
            .min_by_key(|interface| {
                (
                    Some(&interface.name) != default.as_ref(),
                    interface.address.contains(':'),
                )
            })
    } else {
        interfaces
            .iter()
            .filter(|interface| interface.name == selected)
            .min_by_key(|interface| interface.address.contains(':'))
    }
    .cloned();
    Ok(Network {
        interfaces,
        selected: chosen,
    })
}
