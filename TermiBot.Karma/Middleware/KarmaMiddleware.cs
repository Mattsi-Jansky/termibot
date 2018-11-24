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
        private string _incomingMessageRegex = @"([^\s]*)(--|\+\+)(?!\b)";
        private string positiveKarmaOperator = "++";

        private KarmaRepositoryPlugin _karmaRepositoryPlugin;
        
        public KarmaMiddleware(IMiddleware next, KarmaRepositoryPlugin karmaRepositoryPlugin) : base(next)
        {
            _karmaRepositoryPlugin = karmaRepositoryPlugin;
            HandlerMappings = new[]
            {
                new HandlerMapping
                {
                    ValidHandles = RegexHandle.For(_incomingMessageRegex),
                    Description = "Allows upvoting and downvoting on things and people with `--` and `++`.",
                    EvaluatorFunc = KarmaHandler,
                    MessageShouldTargetBot = false,
                    VisibleInHelp = false
                }
            };
        }
        
        private IEnumerable<ResponseMessage> KarmaHandler(IncomingMessage message, IValidHandle matchedHandle)
        {
            var matches = Regex.Matches(message.FullText, _incomingMessageRegex);
            
            foreach (Match match in matches)
            {
                var changeRequest = ParseKarmaChange(match.Value);
                yield return HandleKarmaChange(message, changeRequest);
            }
        }

        private ChangeRequest ParseKarmaChange(string matchedText)
        {
            var matchedTextWithoutWhitespace = matchedText.Trim();
            int itemLength = matchedTextWithoutWhitespace.Length - 2;
            var matchedItem = matchedTextWithoutWhitespace.Substring(0, itemLength);
            var karmaOperator = matchedTextWithoutWhitespace.Substring(itemLength, 2);
            var changeAmount = karmaOperator.Equals(positiveKarmaOperator) ? 1 : -1;
            
            return new ChangeRequest(matchedItem, changeAmount);
        }

        private ResponseMessage HandleKarmaChange(IncomingMessage message, ChangeRequest changeRequest)
        {
            try
            {
                _karmaRepositoryPlugin.Update(changeRequest);
                return message.ReplyToChannel(
                    $"{changeRequest.Name} updated, now has {_karmaRepositoryPlugin.Get(changeRequest.Name)}");
            }
            catch (Exception e)
            {
                return message.ReplyToChannel(e.Message + "\n" + e.StackTrace);
            }
        }
    }
}