async function getUpdateMessage(oldVersion) {
    const now = Math.round(Date.now() / 1000);
    let { newVersion = null, lastChecked = 0 } = JSON.parse(
        localStorage.getItem("update-check") || "{}"
    );

    if (now - lastChecked > 86400) {
        const resp = await fetch("https://api.github.com/repos/ravenclaw900/DietPi-Dashboard/tags");
        const json = await resp.json();

        // Remove preceding 'v'
        newVersion = json[0].name.substring(1);

        localStorage.setItem("update-check", JSON.stringify({ newVersion, lastChecked: now }));
    }

    if (
        newVersion.localeCompare(oldVersion, undefined, {
            numeric: true,
        }) === 1
    ) {
        return `New version available: ${newVersion}`;
    }
}

(() => {
    customElements.define(
        "web-terminal",
        class extends HTMLElement {
            async connectedCallback() {
                const term = new Terminal();

                const fitAddon = new FitAddon.FitAddon();
                term.loadAddon(fitAddon);

                term.open(this);

                fetch("/terminal/stream").then(async (res) => {
                    for await (const chunk of res.body) {
                        term.write(chunk)
                    }
                });

                term.onResize((dimensions) => {
                    fetch("/terminal/resize", { method: "POST", body: new URLSearchParams(dimensions) });
                });

                fitAddon.fit()
                window.addEventListener("resize", () => fitAddon.fit(), 50);

                let sendTimeout = null;
                let sendBuf = "";

                // I would MUCH rather stream this, but that's not possible on HTTP/1.1 and doesn't work at all on Firefox
                term.onData((data) => {
                    clearTimeout(sendTimeout);
                    sendBuf += data;
                    // Short debounce to prevent excess requests
                    sendTimeout = setTimeout(() => {
                        fetch("/terminal/write", { method: "POST", body: sendBuf });
                        sendBuf = "";
                    }, 60);
                });
            }
        }
    );

    customElements.define(
        "code-editor",
        class extends HTMLElement {
            connectedCallback() {
                const textarea = this.querySelector("textarea");
                const pre = this.querySelector("pre");

                const highlight = () => {
                    pre.textContent = textarea.value;
                    microlight(pre);
                };

                const autosize = () => {
                    textarea.style.height = "0px";
                    textarea.style.height = textarea.scrollHeight + "px";
                    pre.style.height = textarea.scrollHeight + "px";
                };

                textarea.addEventListener("input", highlight);
                textarea.addEventListener("input", autosize);

                highlight();
                autosize();
            }
        }
    );
})();
