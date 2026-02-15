use hyper::header;
use maud::{DOCTYPE, Markup, PreEscaped, Render, html};

use crate::http::{
    request::{BackendData, ServerRequest},
    response::ServerResponse,
};

macro_rules! send_req {
    ($req:expr, $variant:ident $(($data:expr))?) => {{
        use proto::{backend::ResponseBackendMessage, frontend::RequestFrontendMessage};

        $req.send_backend_req(RequestFrontendMessage::$variant $(($data))?)
            .await
            .map(|resp| match resp {
                ResponseBackendMessage::$variant(resp) => resp,
                _ => unreachable!(),
            })
    }};
}

pub(crate) use send_req;

macro_rules! send_act {
    ($req:expr, $variant:ident $(($data:expr))?) => {{
        use proto::frontend::ActionFrontendMessage;

        $req.send_backend_action(ActionFrontendMessage::$variant $(($data))?).await
    }};
}

pub(crate) use send_act;

fn header(req: &ServerRequest) -> Result<Markup, ServerResponse> {
    let BackendData {
        backend_list,
        current_backend,
    } = req.extract_backends()?;

    Ok(html! {
        header {
            button .nav-toggle
                aria-controls="nav"
                nm-bind="onclick: () => navOpen = !navOpen, ariaExpanded: () => navOpen"
            {
                (Icon::new("fa6-solid-bars").size(30))
            }

            label .backend-switch {
                span { "Backend" }
                select
                    onchange="document.cookie = `backend=${this.value}; MaxAge=999999999`; window.location.reload()"
                {
                    @for backend in backend_list {
                        @let is_current_backend = backend.0 == current_backend.addr;
                        option value=(backend.0) selected[is_current_backend] {
                            (backend.1) " (" (backend.0) ")"
                        }
                    }
                }
            }

            button .msg-btn
                aria-controls="msgs"
                nm-bind="onclick: () => msgsOpen = !msgsOpen, ariaExpanded: () => msgsOpen"
            {
                (Icon::new("fa6-solid-envelope"))
                span .notifier nm-bind="hidden: () => !newMsg" { (Icon::new("fa6-solid-circle").size(12)) }
            }

            span .theme-switch nm-data="isDark: localStorage.getItem('darkMode') === 'true'" nm-bind="
                oninit: () => {
                    const theme = isDark ? 'dark' : 'light';
                    window.__setDashboardTheme?.(theme);
                }
            " {
                button .theme-toggle nm-bind="
                    onclick: () => {
                        isDark = !isDark;
                        localStorage.setItem('darkMode', isDark);
                        const theme = isDark ? 'dark' : 'light';
                        window.__setDashboardTheme?.(theme);
                    }
                " {
                    span nm-bind="hidden: () => isDark" {
                        (Icon::new("fa6-solid-sun"))
                    }
                    span nm-bind="hidden: () => !isDark" {
                        (Icon::new("fa6-solid-moon"))
                    }
                }
            }
        }
        #msgs {
            ul {
                li nm-bind={"textContent: async () => {
                    const msg = await getUpdateMessage('"(config::APP_VERSION)"');
                    newMsg = !!msg;
                    return msg;
                }"} {}
                @if let Some(update) = current_backend.update {
                    li nm-bind="oninit: () => newMsg = true" { "DietPi Update Available: " (update) }
                }
            }
        }
    })
}

fn nav(req: &ServerRequest) -> Markup {
    let current_page = req.path_segments().next().unwrap_or("system");

    html! {
        nav #nav nm-bind="
            onclick: (e) => {
                if (window.matchMedia('(max-width: 980px)').matches && e.target.closest('a')) {
                    navOpen = false;
                }
            }
        " {
            a href="/system" class=(if current_page == "system" { "active" } else { "" }) aria-current=(if current_page == "system" { "page" } else { "false" }) {
                (Icon::new("fa6-solid-gauge"))
                "System"
            }
            a href="/process" class=(if current_page == "process" { "active" } else { "" }) aria-current=(if current_page == "process" { "page" } else { "false" }) {
                (Icon::new("fa6-solid-microchip"))
                "Processes"
            }
            a href="/software" class=(if current_page == "software" { "active" } else { "" }) aria-current=(if current_page == "software" { "page" } else { "false" }) {
                (Icon::new("fa6-solid-database"))
                "Software"
            }
            a href="/service" class=(if current_page == "service" { "active" } else { "" }) aria-current=(if current_page == "service" { "page" } else { "false" }) {
                (Icon::new("fa6-solid-list"))
                "Services"
            }
            a href="/management" class=(if current_page == "management" { "active" } else { "" }) aria-current=(if current_page == "management" { "page" } else { "false" }) {
                (Icon::new("fa6-solid-user"))
                "Management"
            }
            a href="/terminal" class=(if current_page == "terminal" { "active" } else { "" }) aria-current=(if current_page == "terminal" { "page" } else { "false" }) {
                (Icon::new("fa6-solid-terminal"))
                "Terminal"
            }
            a href="/browser" class=(if current_page == "browser" { "active" } else { "" }) aria-current=(if current_page == "browser" { "page" } else { "false" }) {
                (Icon::new("fa6-solid-folder"))
                "File Browser"
            }
        }
    }
}

fn footer() -> Markup {
    html! {
        footer {
            "DietPi Dashboard v" (config::APP_VERSION) " by ravenclaw900"
            a href="https://github.com/ravenclaw900/DietPi-Dashboard" target="_blank" {
                (Icon::new("cib-github").size(32))
            }
        }
    }
}

pub fn template(
    req: &ServerRequest,
    content: Markup,
    persistent_data: &str,
) -> Result<ServerResponse, ServerResponse> {
    let page = if req.is_fixi() {
        content
    } else {
        html! {
            (DOCTYPE)
            html lang="en" {
                head {
                    meta charset="UTF-8";
                    meta name="viewport" content="width=device-width, initial-scale=1";

                    title { "DietPi Dashboard" }

                    script {
                        (PreEscaped(r#"
                            (() => {
                                const root = document.documentElement;
                                const setTheme = (theme) => {
                                    root.dataset.theme = theme;
                                    root.style.colorScheme = theme;

                                    let meta = document.querySelector('meta[name="color-scheme"]');
                                    if (!meta) {
                                        meta = document.createElement('meta');
                                        meta.name = 'color-scheme';
                                        document.head.append(meta);
                                    }
                                    meta.content = theme;
                                };

                                window.__setDashboardTheme = setTheme;
                                let isDark = false;
                                try {
                                    isDark = localStorage.getItem('darkMode') === 'true';
                                } catch (_) {}
                                setTheme(isDark ? 'dark' : 'light');
                            })();
                        "#))
                    }

                    link rel="icon" href="/favicon.svg" type="image/svg+xml";
                    link rel="stylesheet" href="/static/main.css";
                }
                body
                    nm-data="navOpen: window.matchMedia('(min-width: 981px)').matches, msgsOpen: false, newMsg: false,"
                    nm-bind="
                        oninit: () => {
                            const media = window.matchMedia('(max-width: 980px)');
                            media.addEventListener('change', () => {
                                navOpen = !media.matches;
                                msgsOpen = false;
                            });
                        },
                        'class.nav-closed': () => !navOpen,
                        'class.nav-open': () => navOpen,
                        'class.msgs-open': () => msgsOpen
                    "
                {
                    h1 { "DietPi Dashboard" }

                    (header(req)?)

                    (nav(req))
                    button #nav-overlay type="button" aria-label="Close navigation" nm-bind="
                        hidden: () => !navOpen,
                        onclick: () => navOpen = false
                    " {}

                    main nm-data=(persistent_data) {
                        (content)
                    }

                    (footer())

                    script src="/static/main.js" {}
                }
            }
        }
    };

    Ok(ServerResponse::new()
        .header(header::CONTENT_TYPE, "text/html;charset=UTF-8")
        .body(page.into_string()))
}

pub struct Icon {
    name: &'static str,
    size: u8,
}

impl Icon {
    pub fn new(name: &'static str) -> Self {
        Self { name, size: 24 }
    }

    pub fn size(mut self, size: u8) -> Self {
        self.size = size;
        self
    }
}

impl Render for Icon {
    fn render(&self) -> Markup {
        html! {
            svg width=(self.size) height=(self.size) {
                use href={"/static/icons.svg#" (self.name)} {}
            }
        }
    }
}
