using System.Collections.Generic;
using TermiBot.Karma.Plugins;
using Xunit;

namespace TermiBot.Karma.Tests.Plugins
{
    public class ParseNameFromReasonRequestTests
    {
        [Theory]
        [InlineData("karma reason test", "test")]
        [InlineData("karma reason test 10", "test")]
        [InlineData("karma reason test 1000", "test")]
        [InlineData("karma reason test 1000 ", "test")]
        [InlineData("karma reason test    ", "test")]
        [InlineData("karma reason test_2", "test_2")]
        [InlineData("karma reason test_2 22", "test_2")]
        [InlineData("karma reason test_2 22 ", "test_2")]
        [InlineData("karma reason test_22 ", "test_22")]
        public void ShouldGetKarmaEntryNameFromReasonRequest(string request, string expected)
        {
            
            var plugin = new KarmaPlugin();
            string result = plugin.ParseNameFromReasonRequest(request);

            Assert.Equal(expected, result);
        }
    }
}