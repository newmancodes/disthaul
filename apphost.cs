#:sdk Aspire.AppHost.Sdk@13.2.2
#:package Aspire.Hosting.Keycloak@13.2.2-preview.1.26207.2
#:property NoWarn=ASPIRECERTIFICATES001

var builder = DistributedApplication.CreateBuilder(args);

var keycloak = builder.AddKeycloak("keycloak")
    .WithDataVolume();

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
    .WaitFor(keycloak);

builder.Build().Run();