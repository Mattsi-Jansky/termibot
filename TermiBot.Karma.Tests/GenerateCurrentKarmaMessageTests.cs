using Moq;
using Noobot.Core.MessagingPipeline.Request;
using Noobot.Core.MessagingPipeline.Response;
using TermiBot.Karma.Models;
using TermiBot.Karma.Plugins;
using Xunit;

namespace TermiBot.Karma.Tests
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
    }
}