namespace Zilean.Shared.Features.Grpc;

public abstract class BaseGrpcService<TClient>(ILogger logger) where TClient : ClientBase<TClient>
{
    protected CancellationTokenSource? _grpcCts;
    protected TClient? _client;
    protected bool _isInitialized;
    protected ILogger Logger => logger;
    protected abstract string SocketPath { get; }
    protected virtual Task ShutdownClientAsync(TClient client) => Task.CompletedTask;
    public abstract Task StartServer();

    public async Task StopServer()
    {
        await StartServer();

        try
        {
            if (_client is not null)
            {
                await ShutdownClientAsync(_client);
                Logger.LogInformation($"{nameof(TClient)} gRPC server shutdown signal sent");
            }
        }
        catch (Exception ex)
        {
            Logger.LogWarning(ex, $"Failed to shut down gRPC server via {nameof(TClient)}; falling back to cancel token");
        }

        if (_grpcCts is not null)
        {
            try
            {
                await _grpcCts.CancelAsync();
                Logger.LogInformation($"{nameof(TClient)} gRPC server cancellation requested");
            }
            catch (Exception ex)
            {
                Logger.LogError(ex, $"{nameof(TClient)} Failed to cancel gRPC server");
            }
            finally
            {
                _grpcCts.Dispose();
                _grpcCts = null;
            }
        }

        _client = null;
        _isInitialized = false;
    }

    protected async Task PostServerInitialization()
    {
        for (int i = 0; i < 50 && !File.Exists(SocketPath); i++)
        {
            await Task.Delay(100);
        }

        if (!File.Exists(SocketPath))
        {
            throw new InvalidOperationException(
                $"{nameof(TClient)} gRPC server did not start correctly. Socket file {SocketPath} does not exist.");
        }

        var channel = GrpcChannel.ForAddress(
            "http://unix", new()
            {
                HttpHandler = new SocketsHttpHandler
                {
                    ConnectCallback = async (_, ct) =>
                    {
                        var socket = new Socket(AddressFamily.Unix, SocketType.Stream, ProtocolType.Unspecified);
                        await socket.ConnectAsync(new UnixDomainSocketEndPoint(SocketPath), ct);
                        return new NetworkStream(socket, ownsSocket: true);
                    },
                },
            });

        _client = (TClient)Activator.CreateInstance(typeof(TClient), channel)!;
        _isInitialized = true;
    }
}
