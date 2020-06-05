using System;
using System.Collections;
using System.Collections.Generic;
using System.Linq;
using System.Text.RegularExpressions;
using System.Xml.Schema;
using Microsoft.EntityFrameworkCore.Internal;
using Noobot.Core.MessagingPipeline.Request;
using Noobot.Core.MessagingPipeline.Response;
using Noobot.Core.Plugins;
using TermiBot.Karma.Models;

namespace TermiBot.Karma.Plugins
{
    public class KarmaPlugin : IPlugin
    {
        public const string OperatorRegex = @"([^\`\s]{2,})[^\`\s-\+](--|\+\+)(?!\b)";
        private const string ReasonRegex = OperatorRegex +
                                            @"\s(for|because|due to|over|thanks to|since|considering).*$";
        private const string BacktickQuoteRegex = @"\`.*\`";
        private const string PositiveKarmaOperator = "++";
        
        public const string ListKarmaRegex = @"karma list(( \d\d)|( \d))?";
        public const string ListReasonRegex = @"karma reason (.*)(( \d\d)|( \d))?$";
 
        public void Start()
        {
            
        }

        public void Stop()
        {
            
        }
        
        public ChangeRequest ParseKarmaChange(string matchedText)
        {
            int itemLength = matchedText.Length - 2;
            var matchedItem = matchedText.Substring(0, itemLength);
            var karmaOperator = matchedText.Substring(itemLength, 2);
            var changeAmount = karmaOperator.Equals(PositiveKarmaOperator) ? 1 : -1;
            
            return new ChangeRequest(matchedItem, changeAmount);
        }

        public int ParseNumberFromEndOfRequest(string matchedText)
        {
            var numberRegex = Regex.Match(matchedText, @"\d+$");
            if (!numberRegex.Success) return 10;
            
            var result = Int32.Parse(numberRegex.Value);
            return result == 0 ? 10 : result;
        }

        public String ParseNameFromReasonRequest(string matchedText)
        {
            var beginningOfNameMatch = Regex.Match(matchedText, @"(<)?@[^ ]+ karma reason ");
            var indexOfBeginningOfName = beginningOfNameMatch.Length;
            
            var nameToEnd = matchedText.Substring(indexOfBeginningOfName);
            return nameToEnd.Contains(" ") ? nameToEnd.Substring(0, nameToEnd.IndexOf(" ")) : nameToEnd;
        }
        
        public ChangeRequest ParseKarmaChangeWithReason(string matchedText)
        {
            var operatorMatch = GetOperatorMatchesInMessage(matchedText).First();
            var changeRequest = ParseKarmaChange(operatorMatch.Value);
            var reason = matchedText.Substring(operatorMatch.Length + 1);
            changeRequest = new ChangeRequest(changeRequest, reason);
            
            return changeRequest;
        }

        public IList<Match> GetOperatorMatchesInMessage(string message)
        {
            var inlineCodeMatches = Regex.Matches(message, BacktickQuoteRegex, RegexOptions.IgnoreCase | RegexOptions.Multiline);
            var karmaPhraseMatches = Regex.Matches(message, OperatorRegex, RegexOptions.IgnoreCase | RegexOptions.Multiline);
            return karmaPhraseMatches.Where(x => !IsIntersectingMatchInOtherCollection(x, inlineCodeMatches)).ToList();
        }

        public IList<Match> GetReasonMatchesInMessage(string message)
        {
            var inlineCodeMatches = Regex.Matches(message, BacktickQuoteRegex, RegexOptions.IgnoreCase | RegexOptions.Multiline);
            var karmaPhraseMatches = Regex.Matches(message, ReasonRegex, RegexOptions.IgnoreCase | RegexOptions.Multiline);
            return karmaPhraseMatches.Where(x => !IsIntersectingMatchInOtherCollection(x, inlineCodeMatches)).ToList();
        }

        private bool IsIntersectingMatchInOtherCollection(Match match, MatchCollection inlineCodeMatches)
        {
            foreach (Match inlineCodeMatch in inlineCodeMatches)
            {
                if (IsIntersectingRange(inlineCodeMatch.Index, inlineCodeMatch.Length,
                    match.Index, match.Length)) return true;
            }

            return false;
        }

        private bool IsIntersectingRange(int aIndex, int aLength, int bIndex, int bLength)
        {
            int aEnd = aIndex + aLength;
            int bEnd = bIndex + bLength;

            if (bIndex > aIndex && bIndex > aEnd) return false;
            if (bIndex < aIndex && bEnd < aIndex) return false;
            return true;
        }
        
        public string GenerateCurrentKarmaMessage(ChangeRequest changeRequest,
            int currentKarma)
        {
            string emoji = changeRequest.Amount > 0 ? ":upboat:" : ":downboat:";
            var karmaEntryName = changeRequest.Name.Replace("_"," ");
            var reason = !string.IsNullOrEmpty(changeRequest.Reason) ? $" {changeRequest.Reason}" : string.Empty;
            
            return $"{emoji} {karmaEntryName}: {currentKarma}{reason}";
        }

        public string GenerateCurrentKarmaMessage(Entry entry)
        {
            var karmaEntryName = entry.DisplayName.Replace("_"," ");
            return $"{karmaEntryName}: {entry.Karma}";
        }

        public string GenerateReasonMessage(Reason reason)
        {
            var emoji = reason.Change > 0 ? ":upboat:" : ":downboat:";
            return $"{emoji} {reason.Name} {reason.Value}";
        }
    }
}