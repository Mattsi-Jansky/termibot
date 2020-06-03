using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Logging.Console;
using Noobot.Core;
using Noobot.Core.Configuration;
using Noobot.Core.DependencyResolution;
using TermiBot.Console.Configuration;

namespace Noobot.Examples.ConsoleService
{
    using Console = System.Console;
    
    public class NoobotHost
    {
        private readonly IConfigReader _configReader;
        private INoobotCore _noobotCore;
        private readonly IConfiguration _configuration;

        public NoobotHost(IConfigReader configReader)
        {
            _configReader = configReader;
            _configuration = new TermibotConfiguration();
        }

        public void Start()
        {
            var loggerFactory = LoggerFactory.Create(builder => {
                    builder.AddFilter("Microsoft", LogLevel.Warning)
                        .AddFilter("System", LogLevel.Warning)
                        .AddFilter("SampleApp.Program", LogLevel.Debug)
                        .AddConsole();
                }
            );
            IContainerFactory containerFactory = new ContainerFactory(_configuration, _configReader, loggerFactory.CreateLogger<string>());
            INoobotContainer container = containerFactory.CreateContainer();
            _noobotCore = container.GetNoobotCore();

            Console.WriteLine("Connecting...");
            _noobotCore
                .Connect()
                .ContinueWith(task =>
                {
                    if (!task.IsCompleted || task.IsFaulted)
                    {
                        Console.WriteLine($"Error connecting to Slack: {task.Exception}");
                    }
                })
                .Wait();
        }

        public void Stop()
        {
            Console.WriteLine("Disconnecting...");
            _noobotCore?.Disconnect();
        }
    }
}