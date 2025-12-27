use maud::html;
use serde::{Deserialize, Serialize};

use crate::http::{query_array::QueryArray, request::ServerRequest, response::ServerResponse};

use super::template::{send_req, template};

mod fragments;
mod graph;

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SystemQuery {
    cpu: QueryArray,
    temp: QueryArray,
    ram: QueryArray,
    swap: QueryArray,
    sent: QueryArray,
    recv: QueryArray,
}

pub async fn page(req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
    req.check_login()?;

    let mut query: SystemQuery = req.extract_query()?;

    let cpu_data = send_req!(req, Cpu)?;
    let temp_data = send_req!(req, Temp)?;
    let mem_data = send_req!(req, Mem)?;
    let disk_data = send_req!(req, Disk)?;
    let net_data = send_req!(req, NetIO)?;

    let cpu_meters = fragments::cpu_meters(&cpu_data, &temp_data);
    let mem_meters = fragments::mem_meters(&mem_data);
    let disk_meters = fragments::disk_meters(&disk_data);

    let cpu_graph = fragments::cpu_graph(&cpu_data, &mut query.cpu);
    let temp_graph = fragments::temp_graph(&temp_data, &mut query.temp);
    let mem_graph = fragments::mem_graph(&mem_data, &mut query.ram, &mut query.swap);
    let net_graph = fragments::net_graph(&net_data, &mut query.sent, &mut query.recv);

    let content = html! {
            div #system-swap .card-grid
                nm-bind="oninit: () => $debounce(() => $get('/system'), 2000)"
                data-cpu=(query.cpu)
                data-ram=(query.ram)
                data-swap=(query.swap)
                data-temp=(query.temp)
                data-sent=(query.sent)
                data-recv=(query.recv)
            {
                (cpu_meters)
                (cpu_graph)
                @if let Some(temp_graph) = temp_graph {
                    (temp_graph)
                }
                (mem_meters)
                (mem_graph)
                (disk_meters)
                (net_graph)
            }
    };

    template(&req, content, "x: null, idx: 0")
}
