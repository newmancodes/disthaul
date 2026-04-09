#:sdk Aspire.AppHost.Sdk@13.2.2
#:package Aspire.Hosting.Keycloak@13.2.2-preview.1.26207.2

var builder = DistributedApplication.CreateBuilder(args);

var keycloak = builder.AddKeycloak("keycloak")
    .WithDataVolume();

builder.Build().Run();
