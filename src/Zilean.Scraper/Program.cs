var builder = Host.CreateApplicationBuilder(args);

builder.Configuration.AddConfigurationFiles();

builder.AddOtlpServiceDefaults();

builder.Services.AddAnsiConsole();

builder.Services.AddScrapers(builder.Configuration);
builder.Services.AddSingleton<DefaultCommand>();
builder.Services.AddSingleton<DmmSyncCommand>();
builder.Services.AddSingleton<GenericSyncCommand>();
builder.Services.AddSingleton<ResyncImdbCommand>();

var app = builder.Build();
await app.StartAsync();
return await app.ExecuteCommandLine(args);

