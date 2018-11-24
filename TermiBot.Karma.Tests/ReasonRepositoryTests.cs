using System;
using System.Linq;
using TermiBot.Karma.Models;
using TermiBot.Karma.Persistence;
using Xunit;

namespace TermiBot.Karma.Tests
{
    public class ReasonRepositoryTests : BaseKarmaContextTests
    {
        private string _testName = "testName";

        [Fact]
        public void ShouldStoreReason()
        {
            var repository = CreateRepository();

            repository.Add(new Reason(_testName, 1, "for being the best"));
            var result = repository.Get(_testName);
            
            Assert.NotNull(result);
            Assert.Single(result);
        }

        private ReasonRepository CreateRepository()
        {
            InitContext();
            return new ReasonRepository(_context);
        }
    }
}