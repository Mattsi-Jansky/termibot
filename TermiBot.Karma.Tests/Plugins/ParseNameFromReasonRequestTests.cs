using System.Collections.Generic;
using TermiBot.Karma.Plugins;
using Xunit;

namespace TermiBot.Karma.Tests.Plugins
{
    public class ParseNameFromReasonRequestTests
    {
        [Theory]
        [InlineData("@bot karma reason test", "test")]
        [InlineData("@bot karma reason test 10", "test")]
        [InlineData("@bot karma reason test 1000", "test")]
        [InlineData("@bot karma reason test 1000 ", "test")]
        [InlineData("@bot karma reason test    ", "test")]
        [InlineData("@bot karma reason test_2", "test_2")]
        [InlineData("@bot karma reason test_2 22", "test_2")]
        [InlineData("@bot karma reason test_2 22 ", "test_2")]
        [InlineData("@bot karma reason test_22 ", "test_22")]
        [InlineData("@bot123 karma reason test", "test")]
        [InlineData("@bot123 karma reason test 10", "test")]
        [InlineData("@bot321 karma reason test 1000", "test")]
        [InlineData("@bot32 karma reason test 1000 ", "test")]
        [InlineData("@bot1 karma reason test    ", "test")]
        [InlineData("@bot22 karma reason test_2", "test_2")]
        [InlineData("@bot11 karma reason test_2 22", "test_2")]
        [InlineData("@bot32 karma reason test_2 22 ", "test_2")]
        [InlineData("@bot_999 karma reason test_22 ", "test_22")]
        [InlineData("@noobot-test2 karma reason time", "time")]
        [InlineData("<@UCLJPMP9A> karma reason time", "time")]
        public void ShouldGetKarmaEntryNameFromReasonRequest(string request, string expected)
        {
            var plugin = new KarmaPlugin();
            string result = plugin.ParseNameFromReasonRequest(request);

            Assert.Equal(expected, result);
        }
    }
}