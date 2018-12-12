using System.Collections.Generic;
using TermiBot.Karma.Models;
using TermiBot.Karma.Plugins;
using Xunit;

namespace TermiBot.Karma.Tests.Plugins
{
    public class ParseKarmaChangeWithReasonTests
    {
        [Fact]
        public void WhenGeneratingChangeRequest_ShouldNotIncludeOperator()
        {
            string expected = "test";
            var plugin = new KarmaPlugin();
            var result = plugin.ParseKarmaChangeWithReason("test++ for test");

            Assert.Equal(expected, result.Name);
        }

        [Fact]
        public void WhenUsingPositiveKarmaOperator_ShouldHaveChangeAmoutOne()
        {
            var plugin = new KarmaPlugin();
            
            var result = plugin.ParseKarmaChangeWithReason("test++ for test");
            
            Assert.Equal(1, result.Amount);
        }

        [Fact]
        public void WhenUsingNegativeKarmaOperator_ShouldHaveChangeAmoutNegativeOne()
        {
            var plugin = new KarmaPlugin();
            
            var result = plugin.ParseKarmaChangeWithReason("test-- for test");
            
            Assert.Equal(-1, result.Amount);
        }

        [Fact]
        public void ShouldFillReasonField()
        {
            string expected = "for test";
            var plugin = new KarmaPlugin();
            
            var result = plugin.ParseKarmaChangeWithReason("test-- for test");

            Assert.Equal(expected, result.Reason);
        }
    }
}