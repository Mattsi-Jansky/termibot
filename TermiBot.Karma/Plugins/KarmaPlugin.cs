using System;
using System.Collections.Generic;
using System.Text.RegularExpressions;
using Noobot.Core.MessagingPipeline.Request;
using Noobot.Core.MessagingPipeline.Response;
using Noobot.Core.Plugins;
using TermiBot.Karma.Models;

namespace TermiBot.Karma.Plugins
{
    public class KarmaPlugin : IPlugin
    {
        public static string IncomingMessageRegex = @"([^\s]*)(--|\+\+)(?!\b)";
        private string positiveKarmaOperator = "++";
        
        public void Start()
        {
            
        }

        public void Stop()
        {
            
        }
        
        public ChangeRequest ParseKarmaChange(string matchedText)
        {
            var matchedTextWithoutWhitespace = matchedText.Trim();
            int itemLength = matchedTextWithoutWhitespace.Length - 2;
            var matchedItem = matchedTextWithoutWhitespace.Substring(0, itemLength);
            var karmaOperator = matchedTextWithoutWhitespace.Substring(itemLength, 2);
            var changeAmount = karmaOperator.Equals(positiveKarmaOperator) ? 1 : -1;
            
            return new ChangeRequest(matchedItem, changeAmount);
        }

        public MatchCollection GetMessageMatches(string message)
        {
            return Regex.Matches(message, IncomingMessageRegex);
        }
        
        public string GenerateCurrentKarmaMessage(ChangeRequest changeRequest,
            int currentKarma)
        {
            return $":upboat: {changeRequest.Name}: {currentKarma}";
        }
    }
}