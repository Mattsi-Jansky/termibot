using TermiBot.Karma.Models;
using TermiBot.Karma.Plugins;
using Xunit;

namespace TermiBot.Karma.Tests.Plugins
{
    public class GenerateCurrentKarmaMessageTests
    {
        private string _testName = "test";

        [Fact]
        public void WhenPositiveChange_ShouldGenerateUpboatMessage()
        {
            string expected = $":upboat: {_testName}: 1";
            KarmaPlugin plugin = new KarmaPlugin();
            
            ChangeRequest testChangeRequest = new ChangeRequest(_testName, 1);
            var result = plugin.GenerateCurrentKarmaMessage(testChangeRequest, 1);
            
            Assert.Equal(expected, result);
        }

        [Fact]
        public void WhenNegativeChange_ShouldgenerateDownboatMessage()
        {
            string expected = $":downboat: {_testName}: -1";
            KarmaPlugin plugin = new KarmaPlugin();
            
            ChangeRequest testChangeRequest = new ChangeRequest(_testName, -1);
            var result = plugin.GenerateCurrentKarmaMessage(testChangeRequest, -1);
            
            Assert.Equal(expected, result);
        }
    }
}