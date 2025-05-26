namespace Zilean.Shared.Features.Grpc;

public static class ServiceCollectionExtensions
{
    public static IServiceCollection AddGrpcSupport(this IServiceCollection services)
    {
        services.AddSingleton<IRustGrpcService, RustGrpcService>();
        return services;
    }
}
