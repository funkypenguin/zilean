// using TorrentParserClient = Zilean.Proto.TorrentParser.TorrentParser.TorrentParserClient;
//
// namespace Zilean.Shared.Features.ParseTorrentNames;
//
// public class GoTorrentNameService(ILogger<TorrentParserClient> logger, ZileanConfiguration configuration)
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
//         _ = Cli.Wrap("/Users/prom3theu5/git/promknight/zilean/src/GoPttServer/GoPttServer")
//             .WithEnvironmentVariables(env =>
//             {
//                 var environmentVariables = new Dictionary<string, string?>
//                 {
//                     ["ZILEAN_PTT_GRPC_SOCKET"] = SocketPath,
//                     ["ZILEAN_PTT_WORKER_COUNT"] = "4",
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
