using Noobot.Core.Configuration;
using Jansk.Karma.Middleware;
using Jansk.Karma.Plugins;

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