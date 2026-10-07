import AppKit
import WebKit

/// Implemented by the app: answers window.odomouse.* calls from a page.
protocol BridgeHost: AnyObject {
    func handle(method: String, args: [Any], from page: WebPage, reply: @escaping (String) -> Void)
}

/// One of the shared web UI pages (src/ui) in a WKWebView, created only
/// while it is on screen so the app stays light when idle.
final class WebPage: NSObject, WKScriptMessageHandler, WKNavigationDelegate {
    let webView: WKWebView
    private weak var host: BridgeHost?
    private let pageURL: URL

    /// `tab`: initial dashboard tab. It goes in through a script, not a
    /// `?tab=` query: WKWebView refuses file URLs that carry a query.
    init(page: String, tab: String? = nil, theme: String = "system", host: BridgeHost, size: NSSize) {
        self.host = host
        let resources = Bundle.main.resourceURL!
        let url = resources.appendingPathComponent("ui", isDirectory: true).appendingPathComponent(page)
        pageURL = url

        var boot = "window.ODOMOUSE_PLATFORM = 'mac'; window.ODOMOUSE_THEME = \(JSON.quote(theme));"
        if let tab = tab { boot += " window.ODOMOUSE_TAB = \(JSON.quote(tab));" }
        let content = WKUserContentController()
        content.addUserScript(WKUserScript(source: boot, injectionTime: .atDocumentStart, forMainFrameOnly: true))
        let config = WKWebViewConfiguration()
        config.userContentController = content
        config.websiteDataStore = .nonPersistent()
        webView = WKWebView(frame: NSRect(origin: .zero, size: size), configuration: config)
        super.init()

        content.add(WeakMessageHandler(self), name: "odomouse")
        webView.navigationDelegate = self
        webView.setValue(false, forKey: "drawsBackground") // no white flash before the CSS paints
        webView.allowsMagnification = false
        webView.loadFileURL(url, allowingReadAccessTo: resources)
    }

    /// Break the message-handler link and free the web content process.
    func close() {
        webView.configuration.userContentController.removeScriptMessageHandler(forName: "odomouse")
        webView.navigationDelegate = nil
        webView.stopLoading()
        webView.loadHTMLString("", baseURL: nil)
        webView.removeFromSuperview()
    }

    /// window.__odomouseEmit(name, payload); `json` must be valid JSON.
    func emit(_ name: String, _ json: String) {
        webView.evaluateJavaScript("window.__odomouseEmit && window.__odomouseEmit(\(JSON.quote(name)), \(json))", completionHandler: nil)
    }

    // MARK: WKScriptMessageHandler

    func userContentController(_ controller: WKUserContentController, didReceive message: WKScriptMessage) {
        guard let body = message.body as? [String: Any],
              let id = (body["id"] as? NSNumber)?.intValue,
              let method = body["method"] as? String else { return }
        let args = body["args"] as? [Any] ?? []
        guard let host = host else { return }
        host.handle(method: method, args: args, from: self) { [weak self] json in
            self?.webView.evaluateJavaScript("window.__odomouseReply(\(id), \(json))", completionHandler: nil)
        }
    }

    // MARK: WKNavigationDelegate: the page may never navigate away

    func webView(_ webView: WKWebView, decidePolicyFor action: WKNavigationAction,
                 decisionHandler: @escaping (WKNavigationActionPolicy) -> Void) {
        let url = action.request.url
        let allowed = url?.isFileURL == true && url?.path == pageURL.path
        decisionHandler(allowed ? .allow : .cancel)
    }

    func webViewWebContentProcessDidTerminate(_ webView: WKWebView) {
        webView.loadFileURL(pageURL, allowingReadAccessTo: Bundle.main.resourceURL!)
    }
}

/// WKUserContentController retains its handlers; this keeps WebPage free.
private final class WeakMessageHandler: NSObject, WKScriptMessageHandler {
    weak var target: WKScriptMessageHandler?
    init(_ target: WKScriptMessageHandler) { self.target = target }
    func userContentController(_ c: WKUserContentController, didReceive m: WKScriptMessage) {
        target?.userContentController(c, didReceive: m)
    }
}
