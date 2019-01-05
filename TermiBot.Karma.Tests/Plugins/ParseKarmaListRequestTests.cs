using TermiBot.Karma.Plugins;
using Xunit;

namespace TermiBot.Karma.Tests.Plugins
{
    public class ParseKarmaListRequestTests
    {
        [Fact]
        public void WhenNoNumberGiven_ShouldDefaultToTen()
        {
            var input = "@termibot karma list";

            var plugin = new KarmaPlugin();
            int result =  plugin.ParseKarmaListRequest(input);
            
            Assert.Equal(10, result);
        }

        [Theory]
        [InlineData("5",5)]
        [InlineData("1",1)]
        [InlineData("9",9)]
        [InlineData("55",55)]
        [InlineData("99",99)]
        public void WhenNumberGiven_ShouldReturnNumber(string inputNumber, int expected)
        {
            var input = $"@termibot karma list {inputNumber}";
            
            var plugin = new KarmaPlugin();
            var result =  plugin.ParseKarmaListRequest(input);
            
            Assert.Equal(expected, result);
        }

        [Fact]
        public void WhenZeroGivenShouldReturnDefault()
        {
            var input = "@termibot karma list 0";

            var plugin = new KarmaPlugin();
            int result =  plugin.ParseKarmaListRequest(input);
            
            Assert.Equal(10, result);
        }
    }
}