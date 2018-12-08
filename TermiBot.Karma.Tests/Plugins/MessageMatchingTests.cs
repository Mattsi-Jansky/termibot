using System;
using System.Collections.Generic;
using System.Linq;
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
                "Test++",
                "Test--",
                " Test-- ",
                " Test++ ",
                " Test++ words words",
                "words words Test++ ",
                "I hate microsoft-- for doing bad things",
                "I love microsoft++ because they do good things",
                ":emoji:++",
                " :emoji:++",
                ":emoji:++ ",
                " :emoji:++ ",
                ":emoji:--",
                " :emoji:--",
                ":emoji:-- ",
                " :emoji:-- ",
                "Test---", //In this case, expected behaviour is to match "Test--".
                "Test+++",
                "this++ is a matching phrase",
                "this is a matching phrase++"
            );
        }

        public static IEnumerable<object[]> GetInvalidInputs()
        {
            return GenerateArgumentsFromInputs(
                "Test",
                " Test ",
                "TestPlusPlus",
                "TestMinusMinus",
                "I hate microsoft for doing bad things",
                "I love microsoft because they do good things",
                "++",
                "--",
                "+++",
                "---",
                "--+",
                "++-",
                "this ++is not a matching phrase",
                "this++is not a matchin phrase",
                "++this is not a matching phrase",
                "-++-",
                "+--+",
                "I think C++ is an okay language",
                "`preformatted++`",
                "`so preformatted++`",
                "`preformatted++ to the max`",
                "`preformatted`++",
                "`this doesn't++ have any preformatted tags itself but is inside preformatted tags`"
            );
        }
        
        [Theory]
        [MemberData(nameof(GetValidInputs))]
        public void ShouldMatchValidInputs(string input)
        {
            int expected = 1;
            var plugin = new KarmaPlugin();
            
            var matches = plugin.GetMessageMatches(input);
            int numberOfMatches = matches.Count;
            
            Assert.Equal(expected, numberOfMatches);
            output.WriteLine($"Matched {matches[0].Value}");
        }

        [Theory]
        [MemberData(nameof(GetInvalidInputs))]
        public void ShouldNotMatchInvalidInputs(string input)
        {
            int expected = 0;
            var plugin = new KarmaPlugin();
            
            var matches = plugin.GetMessageMatches(input);
            int numberOfMatches = matches.Count;
            
            Assert.Equal(expected, numberOfMatches);
        }
        
        [Fact]
        public void GivenHyphenatedPhrase_ShouldTreatNormally()
        {
            int expectedCount = 2;
            var expectedMatchvalueOne = "hyphenated-word--";
            var expectedMatchvalueTwo = "part-time-teachers++";
            var plugin = new KarmaPlugin();
            
            var matches = plugin.GetMessageMatches("hyphenated-word-- part-time-teachers++");
            int numberOfMatches = matches.Count;
            
            Assert.Equal(expectedCount, numberOfMatches);
            Assert.Equal(expectedMatchvalueOne, matches[0].Value);
            Assert.Equal(expectedMatchvalueTwo, matches[1].Value);
        }

        private static object[] GenerateArgumentsFromInput(string input) { return new object[] { input };}

        private static IEnumerable<object[]> GenerateArgumentsFromInputs(params string[] inputs)
        {
            return inputs.Select(GenerateArgumentsFromInput);
        }
    }
}