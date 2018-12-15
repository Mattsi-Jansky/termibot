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
        public void WhenNegativeChange_ShouldGenerateDownboatMessage()
        {
            string expected = $":downboat: {_testName}: -1";
            KarmaPlugin plugin = new KarmaPlugin();
            
            ChangeRequest testChangeRequest = new ChangeRequest(_testName, -1);
            var result = plugin.GenerateCurrentKarmaMessage(testChangeRequest, -1);
            
            Assert.Equal(expected, result);
        }

        [Fact]
        public void WhenNameIncludesUnderscore_ShouldDisplaySpaceInPlaceOfUnderscore()
        {
            string expected = ":downboat: Bob Ross: -1";
            KarmaPlugin plugin = new KarmaPlugin();
            
            ChangeRequest testChangeRequest = new ChangeRequest("Bob_Ross", -1);
            var result = plugin.GenerateCurrentKarmaMessage(testChangeRequest, -1);
            
            Assert.Equal(expected, result);
        }

        [Fact]
        public void WhenReasonIncluded_ShouldAddReasonToMessage()
        {
            string expected = ":upboat: BobRoss: 1 for test";
            var plugin = new KarmaPlugin();
            
            ChangeRequest testChangeRequest = new ChangeRequest("BobRoss", 1, "for test");
            var result = plugin.GenerateCurrentKarmaMessage(testChangeRequest, 1);

            Assert.Equal(expected, result);
        }

        [Fact]
        public void GivenEntry_ShouldGenerateMessage()
        {
            string expected = "test: 9";
            KarmaPlugin plugin = new KarmaPlugin();
            
            var testEntry = new Entry(_testName, 9);
            var result = plugin.GenerateCurrentKarmaMessage(testEntry);
            
            Assert.Equal(expected, result);
        }

        [Fact]
        public void GivenEntry_WhenNameIncludesUnderscore_ShouldDisplaySpaceInPlaceOfUnderscore()
        {
            string expected = "Bob Ross: -1";
            KarmaPlugin plugin = new KarmaPlugin();
            
            var testEntry = new Entry("Bob_Ross", -1);
            var result = plugin.GenerateCurrentKarmaMessage(testEntry);
            
            Assert.Equal(expected, result);
        }
    }
}