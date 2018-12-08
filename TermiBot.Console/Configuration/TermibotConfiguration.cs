using Noobot.Core.Configuration;
using Noobot.Toolbox.Middleware;
using Noobot.Toolbox.Plugins;
using TermiBot.Karma.Middleware;
using TermiBot.Karma.Plugins;

namespace TermiBot.Console.Configuration
{

    public class TermibotConfiguration : ConfigurationBase
    {
        public TermibotConfiguration()
        {
            UseMiddleware<KarmaMiddleware>();
            
            UsePlugin<KarmaRepositoryPlugin>();
            UsePlugin<KarmaPlugin>();
        }
    }
}