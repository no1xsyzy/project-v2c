use "encode/base64"
use "files"
use "json"

use "http"
use "net_ssl"
// use "http_server"


actor Main
  new create(env: Env) =>
    let port = "8423"
    let host = "localhost"
    let limit: USize = 100

    let logger = CommonLog(env.out)

    let auth = try
      env.root as AmbientAuth
    else
      env.out.print("unable to use network")
      return
    end

    HTTPServer(
      auth,
      ListenHandler(env),
      BackendMaker(env),
      logger
      where service=port, host=host, limit=limit, reversedns=auth
    )


class ListenHandler
  let _env: Env

  new iso create(env: Env) =>
    _env = env

  fun ref listening(server: HTTPServer ref) =>
    try
      (let host, let service) = server.local_address().name()?
      _env.err.print("Listening at " + host + ":" + service)
    else
      _env.err.print("Couldn't get local address.")
      _env.exitcode(1)
      server.dispose()
    end

  fun ref not_listening(server: HTTPServer ref) =>
    _env.err.print("Failed to listen.")
    _env.exitcode(1)

  fun ref closed(server: HTTPServer ref) =>
    _env.err.print("Shutdown.")


class BackendMaker is HandlerFactory
  let _env: Env

  new val create(env: Env) =>
    _env = env

  fun apply(session: HTTPSession): HTTPHandler^ => BackendHandler(_env, session)


class BackendHandler is HTTPHandler
  let _env: Env
  let _session: HTTPSession

  new ref create(env: Env, session: HTTPSession) =>
    _env = env
    _session = session

  fun ref apply(request: Payload val) =>
    match (request.method, request.url.path, request.url.query)
    | ("GET", "/v2rayn_to_clash", let query: String) =>
        var upstream_url: (String | None) = None
        for kv in query.split("&").values() do
          let kvp = kv.split("=", 2)
          let k = try kvp(0)? else "rubbish key" end
          let v = try kvp(1)? else "rubbish value" end
          if k == "from" then upstream_url = v end
        end
        match upstream_url
        | let upstream_url': String => UpstreamFetcher(_env, upstream_url', _session)
        else
          _session(recover val Payload.response().>add_chunk("specify upstream URL with `from`") end)
        end
    else
        _session(recover val Payload.response().>add_chunk("not found") end)
    end
  fun ref finished() => None


actor UpstreamFetcher
  let _env: Env
  let _session: HTTPSession

  var _collected: String ref

  new create(env: Env, upstream_url: String, session: HTTPSession) =>
    _env = env
    _session = session
    _collected = String

    let sslctx = try
      recover
        SSLContext
          .>set_client_verify(true)
          .>set_authority(FilePath(_env.root as AmbientAuth, "cacert.pem")?)?
      end
    end

    let client = try
      HTTPClient(env.root as AmbientAuth, consume sslctx)
    else
      _session(recover val Payload.response().>add_chunk("unable to use network") end)
      return
    end

    let dumpMaker = recover val UpstreamNotifyFactory.create(this) end

    let url = try
      URL.valid(URLEncode.decode(upstream_url)?)?
    else
      _session(recover val Payload.response().>add_chunk("Invalid URL `" + upstream_url + "`") end)
      return
    end

    let req = Payload.request("GET", url)
    req("User-Agent") = "v2c-pony"

    try
      let sentreq = client(consume req, dumpMaker)?
    else
      _session(recover val Payload.response().>add_chunk("Malformed URL `" + upstream_url + "`") end)
    end

  be cancelled() =>
    _session(recover val Payload.response(StatusServiceUnavailable).>add_chunk("upstream cancelled") end)

  be failed(reason: HTTPFailureReason) =>
    let response: Payload = Payload.response(StatusServiceUnavailable)
    match reason
    | AuthFailed =>
      response.add_chunk("auth failed")
    | ConnectFailed =>
      response.add_chunk("connect failed")
    | ConnectionClosed =>
      response.add_chunk("connect failed")
    end
    _session(consume response)

  be have_response(response: Payload val) =>
    if response.status == 0 then
      _session(recover val Payload.response(StatusServiceUnavailable).>add_chunk("failed") end)
    end

    try
      for piece in response.body()?.values() do
        _collected.append(piece)
      end
      finished()
    end

  be have_body(data: ByteSeq val) =>
    _collected.append(data)

  be finished() =>
    let send = _collected.clone()
    _collected.clear()

    let donce = try
      Base64.decode[String iso](consume send)?
    else
      _session(recover val Payload.response().>add_chunk("Malformed Upstream") end)
      return
    end

    let m: String iso = "proxies:\n".string()

    for line in donce.split("\n").values() do
      _env.out.print("line: " + line)
      try
        let linesplit = line.split_by("://")
        let protocol = linesplit(0)?
        let sin = linesplit(1)?
        _env.out.print("sin>"+sin.string()+"<sin")

        let dtwice = Base64.decode[String iso](consume sin)?
        let doc = JsonDoc
        _env.out.print("dtwice>"+dtwice.string()+"<dtwice")
        doc.parse(consume dtwice)?
        let json: JsonObject = doc.data as JsonObject

        let t: String iso = "".string()
        t.append("- name: \"" + (json.data("ps")? as String) + "\"\n")
        t.append("  type: \"" + consume protocol + "\"\n")
        t.append("  server: \"" + (json.data("add")? as String) + "\"\n")
        t.append("  port: \"" + (json.data("port")? as String) + "\"\n")
        t.append("  uuid: \"" + (json.data("id")? as String) + "\"\n")
        t.append("  alterId: \"" + (json.data("aid")? as String) + "\"\n")
        t.append("  cipher: auto\n")
        t.append("  skip-cert-verify: true\n")
        t.append("  network: " + (json.data("net")? as String) + "\n")
        t.append("  ws-path: " + (json.data("path")? as String) + "\n")
        t.append("  ws-headers:\n")
        t.append("    host: " + (json.data("host")? as String) + "\n")
        try if (json.data("tls")? as String) == "tls" then t.append("  tls: true\n") end end
        m.append(consume t)
      end
    end
    _session(recover val Payload.response().>add_chunk(consume m) end)


class UpstreamNotifyFactory is HandlerFactory
  let _fetcher: UpstreamFetcher

  new iso create(fetcher: UpstreamFetcher) =>
    _fetcher = fetcher

  fun apply(session: HTTPSession): HTTPHandler ref^ => UpstreamHandler(_fetcher, session)


class UpstreamHandler is HTTPHandler
  let _fetcher: UpstreamFetcher
  let _session: HTTPSession

  new ref create(fetcher: UpstreamFetcher, session: HTTPSession) =>
    _fetcher = fetcher
    _session = session

  fun ref apply(response: Payload val) => _fetcher.have_response(response)

  fun ref chunk(data: ByteSeq val) => _fetcher.have_body(data)

  fun ref finished() =>
    _fetcher.finished()
    _session.dispose()

  fun ref cancelled() => _fetcher.cancelled()

  fun ref failed(reason: HTTPFailureReason) => _fetcher.failed(reason)

