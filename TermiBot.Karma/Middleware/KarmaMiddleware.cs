using System;
using System.Collections.Generic;
using System.Text.RegularExpressions;
using Noobot.Core.MessagingPipeline.Middleware;
using Noobot.Core.MessagingPipeline.Middleware.ValidHandles;
using Noobot.Core.MessagingPipeline.Request;
using Noobot.Core.MessagingPipeline.Response;
using TermiBot.Karma.Models;
using TermiBot.Karma.Plugins;

namespace TermiBot.Karma.Middleware
{
        public class KarmaMiddleware : MiddlewareBase
    {

        private KarmaRepositoryPlugin _karmaRepositoryPlugin;
        private KarmaPlugin _karmaPlugin;
        
        public KarmaMiddleware(IMiddleware next, KarmaRepositoryPlugin karmaRepositoryPlugin, KarmaPlugin karmaPlugin) : base(next)
        {
            _karmaRepositoryPlugin = karmaRepositoryPlugin;
            _karmaPlugin = karmaPlugin;
            HandlerMappings = new[]
            {
                new HandlerMapping
                {
                    ValidHandles = RegexHandle.For(KarmaPlugin.IncomingMessageRegex),
                    Description = "Allows upvoting and downvoting on things and people with `--` and `++`.",
                    EvaluatorFunc = KarmaHandler,
                    MessageShouldTargetBot = false,
                    VisibleInHelp = false
                }
            };
        }
        
        private IEnumerable<ResponseMessage> KarmaHandler(IncomingMessage message, IValidHandle matchedHandle)
        {
            var matches = _karmaPlugin.GetMessageMatches(message.FullText);
            
            foreach (Match match in matches)
            {
                var changeRequest = _karmaPlugin.ParseKarmaChange(match.Value);
                yield return HandleKarmaChange(message, changeRequest);
            }
        }
        
        private ResponseMessage HandleKarmaChange(IncomingMessage message, ChangeRequest changeRequest)
        {
            try
            {
                _karmaRepositoryPlugin.Update(changeRequest);
                var currentKarma = _karmaRepositoryPlugin.Get(changeRequest.Name);
                return message.ReplyToChannel(_karmaPlugin.GenerateCurrentKarmaMessage(changeRequest, currentKarma));
            }
            catch (Exception e)
            {
                return ResponseMessage.ChannelMessage("bots",e.Message + "\n" + e.StackTrace,new List<Attachment>());
            }
        }
    }
}