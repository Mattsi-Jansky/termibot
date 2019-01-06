using System.Collections.Generic;
using TermiBot.Karma.Models;
using TermiBot.Karma.Plugins;
using Xunit;

namespace TermiBot.Karma.Tests.Plugins
{
    public class GenerateReasonMessageTests
    {
        [Theory]
        [InlineData("test", "for being rubbish", ":upboat: test for being rubbish")]
        [InlineData("microsoft", "because dotnet is great", ":upboat: microsoft because dotnet is great")]
        public void ShouldReturnReason(string name, string reasonValue, string expected)
        {
            Reason reason = new Reason(name, 1, reasonValue);
            var plugin = new KarmaPlugin();

            string result = plugin.GenerateReasonMessage(reason);

            Assert.Equal(expected, result);
        }
        
        [Theory]
        [InlineData("test", "for being rubbish", ":downboat: test for being rubbish")]
        [InlineData("microsoft", "because dotnet is great", ":downboat: microsoft because dotnet is great")]
        public void WhenChangeNegative_ShouldUseDownboat(string name, string reasonValue, string expected)
        {
            Reason reason = new Reason(name, -1, reasonValue);
            var plugin = new KarmaPlugin();

            string result = plugin.GenerateReasonMessage(reason);

            Assert.Equal(expected, result);
        }
    }
}