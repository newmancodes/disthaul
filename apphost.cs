#:sdk Aspire.AppHost.Sdk@13.2.2
#:package Aspire.Hosting.Kafka@13.2.2
#:package Aspire.Hosting.Keycloak@13.2.2-preview.1.26207.2
#:package Aspire.Hosting.PostgreSQL@13.2.2
#:package Aspire.Hosting.Redis@13.2.2
#:property NoWarn=ASPIRECERTIFICATES001

var builder = DistributedApplication.CreateBuilder(args);

var keycloak = builder.AddKeycloak("keycloak")
    .WithDataVolume();

var kafka = builder.AddKafka("kafka")
    .WithDataVolume()
    .WithKafkaUI();

var redis = builder.AddRedis("redis")
    .WithDataVolume();

var postgres = builder.AddPostgres("postgres");
var clientManagementDatabase = postgres.AddDatabase("client-management");

var clientManagementApi = builder
    .AddExecutable("client-management-api", "cargo", "./client-management", "run", "--package", "api")
    .WithHttpEndpoint(name: "http", env: "SERVER__PORT")
    .WithHttpHealthCheck("/health")
    .WithOtlpExporter()
    .WithReference(kafka)
    .WithReference(keycloak)
    .WithReference(clientManagementDatabase)
    .WithReference(redis)
    .WaitFor(kafka)
    .WaitFor(keycloak)
    .WaitFor(clientManagementDatabase)
    .WaitFor(redis);

var ui = builder
    .AddExecutable("ui", "trunk", "./ui", "serve", "--watch", ".")
    .WithHttpsEndpoint(name: "https", env: "TRUNK_SERVE_PORT")
    .WithHttpsDeveloperCertificate()
    .WithHttpsCertificateConfiguration(ctx =>
    {
        ctx.EnvironmentVariables["TRUNK_SERVE_TLS_CERT_PATH"] = ctx.CertificatePath;
        ctx.EnvironmentVariables["TRUNK_SERVE_TLS_KEY_PATH"] = ctx.KeyPath;
        return Task.CompletedTask;
    })
    .WithOtlpExporter()
    .WithReference(keycloak)
    .WithEnvironment("CLIENT_MANAGEMENT_API_URL", clientManagementApi.GetEndpoint("http")) // TODO: Use https when client management API supports it
    .WaitFor(keycloak)
    .WaitFor(clientManagementApi);

builder.Build().Run();