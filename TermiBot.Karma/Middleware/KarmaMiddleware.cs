using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.RegularExpressions;
using System.Threading.Tasks;
using Flurl.Util;
using Microsoft.EntityFrameworkCore.Internal;
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
                    ValidHandles = RegexHandle.For(KarmaPlugin.OperatorRegex),
                    Description = "Allows upvoting and downvoting on things and people with `--` and `++`.",
                    EvaluatorFunc = KarmaHandler,
                    MessageShouldTargetBot = false,
                    VisibleInHelp = false
                }
            };
        }
        
        private IEnumerable<ResponseMessage> KarmaHandler(IncomingMessage message, IValidHandle matchedHandle)
        {
            var operatorMatches = _karmaPlugin.GetOperatorMatchesInMessage(message.FullText);
            var reasonMatches = _karmaPlugin.GetReasonMatchesInMessage(message.FullText);

            operatorMatches = operatorMatches.Where(x => !reasonMatches.Any(y => y.Index == x.Index)).ToList();
            
            foreach (Match match in operatorMatches)
            {
                var changeRequest = _karmaPlugin.ParseKarmaChange(match.Value);
                yield return HandleKarmaChange(message, changeRequest);
            }
            foreach(Match match in reasonMatches)
            {
                var changeRequest = _karmaPlugin.ParseKarmaChangeWithReason(match.Value);
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