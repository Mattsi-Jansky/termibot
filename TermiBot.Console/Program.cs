using System;
using System.IO;
using System.Reflection;
using System.Threading;
using Noobot.Core;
using Noobot.Core.Configuration;
using Noobot.Examples.ConsoleService;
using TermiBot.Console.Configuration;

namespace TermiBot.Console
{
    using Console = System.Console;
    
    public class Program : Noobot.Console.Program
    {
        private static readonly ManualResetEvent _quitEvent = new ManualResetEvent(false);
        
        static void Main(string[] args)
        {
            Console.WriteLine(
                $"Noobot.Core assembly version: {Assembly.GetAssembly(typeof(INoobotCore)).GetName().Version}");
#if debug
            var host = new NoobotHost(JsonConfigReader.ForAbsolutePath($"{AppDomain.CurrentDomain.BaseDirectory}/Configuration/config.json"));
#else
            var host = new NoobotHost(JsonConfigReader.ForAbsolutePath($"/home/ubuntu/bot/config.json"));
#endif
            
            host.Start();
            _quitEvent.WaitOne();
        }
    }
}