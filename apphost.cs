#:sdk Aspire.AppHost.Sdk@13.2.2
#:package Aspire.Hosting.Keycloak@13.2.2-preview.1.26207.2

using Aspire.Hosting.ApplicationModel;

var builder = DistributedApplication.CreateBuilder(args);

var keycloak = builder.AddKeycloak("keycloak")
    .WithDataVolume();

// How to pass the PORT down such that trunk finds it in env:TRUNK_SERVE_PORT
// How to setup TLS env:TRUNK_SERVE_TLS_KEY_PATH env:TRUNK_SERVE_TLS_CERT_PATH
var ui = builder.AddTrunkApp("ui", "../ui")
    .WithArgs("--watch", ".") // TODO: Only have this for development loop.
    .WithReference(keycloak)
    .WaitFor(keycloak);
    // TODO : Resource fails to start but no useful logs. Need to investigate further.

builder.Build().Run();

#region Community Toolkit Possibilities?

public static class TrunkAppHostingExtension
{
    public static IResourceBuilder<TrunkAppExecutableResource> AddTrunkApp(this IDistributedApplicationBuilder builder, [ResourceName] string name, string workingDirectory, string[]? args = null)
    {
        ArgumentNullException.ThrowIfNull(builder, nameof(builder));
        ArgumentException.ThrowIfNullOrWhiteSpace(name, nameof(name));
        ArgumentException.ThrowIfNullOrWhiteSpace(workingDirectory, nameof(workingDirectory));

        string[] allArgs = args is { Length: > 0 }
            ? ["serve", .. args]
            : ["serve"];

        workingDirectory = Path.Combine(builder.AppHostDirectory, workingDirectory).NormalizePathForCurrentPlatform();
        var resource = new TrunkAppExecutableResource(name, workingDirectory);

        return builder
            .AddResource(resource)
            .WithTrunkDefaults()
            .WithArgs(allArgs);
    }

    private static IResourceBuilder<TrunkAppExecutableResource> WithTrunkDefaults(this IResourceBuilder<TrunkAppExecutableResource> builder) =>
        builder
            .WithOtlpExporter();
}

public class TrunkAppExecutableResource(string name, string workingDirectory)
    : ExecutableResource(name, "trunk", workingDirectory), IResourceWithServiceDiscovery
{
}

internal static class PathNormalizer
{
    public static string NormalizePathForCurrentPlatform(this string path)
    {
        if (string.IsNullOrWhiteSpace(path) == true)
        {
            return path;
        }

        path = path.Replace('\\', Path.DirectorySeparatorChar).Replace('/', Path.DirectorySeparatorChar);

        return Path.GetFullPath(path);
    }
}

#endregion