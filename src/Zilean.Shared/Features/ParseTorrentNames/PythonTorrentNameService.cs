// using TorrentParserClient = Zilean.Proto.TorrentParser.TorrentParser.TorrentParserClient;
//
// namespace Zilean.Shared.Features.ParseTorrentNames;
//
// public class PythonTorrentNameService(ILogger<TorrentParserClient> logger, ZileanConfiguration configuration)
//     : BaseParseTorrentNameService(logger, configuration)
// {
//     public override async Task StartServer()
//     {
//         if (_isInitialized)
//         {
//             return;
//         }
//
//         _grpcCts = new();
//
//         _ = Cli.Wrap("/usr/bin/python3")
//             .WithArguments(["/app/python-ptt-server/main.py"])
//             .WithEnvironmentVariables(env =>
//             {
//                 var environmentVariables = new Dictionary<string, string?>
//                 {
//                     ["PYTHONPATH"] = "/app/python-ptt-server",
//                 };
//
//                 env.Set(environmentVariables);
//             })
//             .WithStandardOutputPipe(PipeTarget.ToDelegate(line => Logger.LogInformation("[zilean-ptt] {Line}", line)))
//             .WithStandardErrorPipe(PipeTarget.ToDelegate(line => Logger.LogError("[zilean-ptt] ERR: {Line}", line)))
//             .ExecuteAsync(_grpcCts.Token);
//
//         await PostServerInitialization();
//     }
// }
