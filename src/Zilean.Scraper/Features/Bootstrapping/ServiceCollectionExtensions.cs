namespace Zilean.Scraper.Features.Bootstrapping;

public static class ServiceCollectionExtensions
{
    public static void AddScrapers(this IServiceCollection services, IConfiguration configuration)
    {
        var zileanConfiguration = configuration.GetZileanConfiguration();

        services.AddHttpClient();
        services.AddSingleton(zileanConfiguration);
        services.AddImdbServices();
        services.AddDmmServices();
        services.AddGenericServices();
        services.AddZileanDataServices(zileanConfiguration);
        services.AddGrpcSupport();
        services.AddHostedService<EnsureMigrated>();
    }

    public static IServiceCollection AddAnsiConsole(this IServiceCollection services)
    {
        Console.OutputEncoding = Encoding.UTF8;

        var settings = new AnsiConsoleSettings
        {
            Ansi = AnsiSupport.Detect,
            Interactive = InteractionSupport.Detect,
            ColorSystem = ColorSystemSupport.Detect,
        };

        var ansiConsole = AnsiConsole.Create(settings);

        services.AddSingleton(ansiConsole);

        return services;
    }

    private static void AddDmmServices(this IServiceCollection services)
    {
        services.AddSingleton<DmmScraping>();
        services.AddTransient<DmmService>();
    }

    private static void AddGenericServices(this IServiceCollection services)
    {
        services.AddSingleton<GenericIngestionScraping>();
        services.AddSingleton<KubernetesServiceDiscovery>();
    }

    private static void AddImdbServices(this IServiceCollection services)
    {
        services.AddSingleton<ImdbConfiguration>();
        services.AddSingleton<ImdbFileService>();
    }
}
