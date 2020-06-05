using System;
using System.Collections.Generic;
using System.Linq;
using Flurl.Util;
using TermiBot.Karma.Plugins;
using Xunit;
using Xunit.Abstractions;

namespace TermiBot.Karma.Tests.Plugins
{
    public class MessageMatchingTests
    {
        private readonly ITestOutputHelper output;

        public MessageMatchingTests(ITestOutputHelper output)
        {
            this.output = output;
        }

        public static IEnumerable<object[]> GetValidInputs()
        {
            return GenerateArgumentsFromInputs(
                ("Test++", "Test++"),
                ("Test--", "Test--"),
                (" Test-- ", "Test--"),
                (" Test++ ", "Test++"),
                (" Test++ words words", "Test++"),
                ("words words Test++ ", "Test++"),
                ("I hate microsoft-- for doing bad things", "microsoft--"),
                ("I love microsoft++ because they do good things", "microsoft++"),
                (":emoji:++", ":emoji:++"),
                (" :emoji:++", ":emoji:++"),
                (":emoji:++ ", ":emoji:++"),
                (" :emoji:++ ", ":emoji:++"),
                (":emoji:--", ":emoji:--"),
                (" :emoji:--", ":emoji:--"),
                (":emoji:-- ", ":emoji:--"),
                (" :emoji:-- ", ":emoji:--"),
                ("Test---", "Test--"),
                ("Test+++", "Test++"),
                ("this++ is a matching phrase", "this++"),
                ("this is a matching phrase++", "phrase++")
            );
        }

        public static IEnumerable<object[]> GetInvalidInputs()
        {
            return GenerateArgumentsFromInputs(
                ("Test", ""),
                (" Test ", ""),
                ("TestPlusPlus", ""),
                ("TestMinusMinus", ""),
                ("I hate microsoft for doing bad things", ""),
                ("I love microsoft because they do good things", ""),
                ("++", ""),
                ("--", ""),
                ("+++", ""),
                ("---", ""),
                ("--+", ""),
                ("++-", ""),
                ("this ++is not a matching phrase", ""),
                ("this++is not a matchin phrase", ""),
                ("++this is not a matching phrase", ""),
                ("-++-", ""),
                ("+--+", ""),
                ("I think C++ is an okay language", ""),
                ("`preformatted++`", ""),
                ("`so preformatted++`", ""),
                ("`preformatted++ to the max`", ""),
                ("`preformatted`++", ""),
                ("`this doesn't++ have any preformatted tags itself but is inside preformatted tags`", "")
            );
        }

        public static IEnumerable<object[]> GetValidReasonInputs()
        {
            return GenerateArgumentsFromInputs(
                ("test++ for test", "test++ for test"),
                ("test-- because test", "test-- because test"),
                ("test++ because test", "test++ because test"),
                ("test++ thanks to test", "test++ thanks to test"),
                ("test-- considering test", "test-- considering test"),
                (" test++ for words", "test++ for words"),
                ("test++ for words ", "test++ for words "), //In this case we don't care about trailing spaces
                ("test++ test++ for words", "test++ for words"),
                ("test++ for test++ for test++ for test++", "test++ for test++ for test++ for test++"),
                ("test++ for reasons" + '\n' + "also something else", "test++ for reasons")
            );
        }

        [Theory]
        [MemberData(nameof(GetValidInputs))]
        public void ShouldMatchValidInputs(string input, string expectedMatch)
        {
            int expected = 1;
            var plugin = new KarmaPlugin();
            
            var matches = plugin.GetOperatorMatchesInMessage(input);
            int numberOfMatches = matches.Count;
            
            Assert.Equal(expected, numberOfMatches);
            Assert.Equal(expectedMatch, matches[0].Value);
        }

        [Theory]
        [MemberData(nameof(GetInvalidInputs))]
        public void ShouldNotMatchInvalidInputs(string input, string expectedMatch)
        {
            int expectedCount = 0;
            var plugin = new KarmaPlugin();
            
            var matches = plugin.GetOperatorMatchesInMessage(input);
            int numberOfMatches = matches.Count;
            
            Assert.Equal(expectedCount, numberOfMatches);
        }

        [Theory]
        [MemberData(nameof(GetValidReasonInputs))]
        public void ShouldMatchValidReasonInputs(string input, string expectedMatch)
        {
            int expectedCount = 1;
            var plugin = new KarmaPlugin();
            
            var matches = plugin.GetReasonMatchesInMessage(input);
            int numberOfMatches = matches.Count;
            
            Assert.Equal(expectedCount, numberOfMatches);
            Assert.Equal(expectedMatch, matches[0].Value);
        }

        public static IEnumerable<object[]> GetInvalidReasonInputs()
        {
            return new List<string[]>
            {
                new[] {"test++for test"},
                new[] {"`test++ for test`"}
            };
        }
        
        [Theory]
        [MemberData(nameof(GetInvalidReasonInputs))]
        public void ShouldNotMatchInvalidReasonInputs(string input)
        {
            int expectedCount = 0;
            var plugin = new KarmaPlugin();
            
            var matches = plugin.GetReasonMatchesInMessage(input);
            int numberOfMatches = matches.Count;
            
            Assert.Equal(expectedCount, numberOfMatches);
        }
        
        [Fact]
        public void GivenHyphenatedPhrase_ShouldTreatNormally()
        {
            int expectedCount = 2;
            var expectedMatchvalueOne = "hyphenated-word--";
            var expectedMatchvalueTwo = "part-time-teachers++";
            var plugin = new KarmaPlugin();
            
            var matches = plugin.GetOperatorMatchesInMessage("hyphenated-word-- part-time-teachers++");
            int numberOfMatches = matches.Count;
            
            Assert.Equal(expectedCount, numberOfMatches);
            Assert.Equal(expectedMatchvalueOne, matches[0].Value);
            Assert.Equal(expectedMatchvalueTwo, matches[1].Value);
        }

        [Fact]
        public void GivenAdditionalPlusesOrMinnuses_ShouldMatchOnlyTwo()
        {
            int expectedCount = 4;
            var expectedMatchvalueOne = "test++";
            var expectedMatchvalueTwo = "test--";
            var plugin = new KarmaPlugin();
            
            var matches = plugin.GetOperatorMatchesInMessage("test+++ test++++++++++ test--- test----------");
            int numberOfMatches = matches.Count;
            
            Assert.Equal(expectedCount, numberOfMatches);
            Assert.Equal(expectedMatchvalueOne, matches[0].Value);
            Assert.Equal(expectedMatchvalueOne, matches[1].Value);
            Assert.Equal(expectedMatchvalueTwo, matches[2].Value);
            Assert.Equal(expectedMatchvalueTwo, matches[3].Value);
        }

        [Fact]
        public void GivenWhitespace_ShouldOnlyMatchCharacters()
        {
            int expectedCount = 1;
            var expectedMatchvalue = "test++";
            var plugin = new KarmaPlugin();
            
            var matches = plugin.GetOperatorMatchesInMessage(" test++ ");
            int numberOfMatches = matches.Count;
            
            Assert.Equal(expectedCount, numberOfMatches);
            Assert.Equal(expectedMatchvalue, matches[0].Value);
        }

        private static object[] GenerateArgumentsFromInput((string, string) input) { return new object[] { input.Item1, input.Item2 };}

        private static IEnumerable<object[]> GenerateArgumentsFromInputs(params (string, string)[] inputs)
        {
            return inputs.Select(GenerateArgumentsFromInput);
        }
    }
}