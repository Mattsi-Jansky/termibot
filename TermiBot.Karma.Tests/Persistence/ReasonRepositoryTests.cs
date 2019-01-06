using System.Linq;
using TermiBot.Karma.Models;
using TermiBot.Karma.Persistence;
using Xunit;

namespace TermiBot.Karma.Tests.Persistence
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

        [Fact]
        public void GivenNoReasonExists_Get_ShouldReturnEmptyEnumerable()
        {
            var repository = CreateRepository();
            
            var result = repository.Get(_testName);

            Assert.NotNull(result);
            Assert.Empty(result);
        }
        
        [Fact]
        public void GivenNumberOfResultsToReturn_Get_ShouldReturnSpecifiedNumberOfResults()
        {
            var repository = CreateRepository();
            
            repository.Add(new Reason(_testName, 1, "for being the best"));
            repository.Add(new Reason(_testName, 1, "for being the best"));
            repository.Add(new Reason(_testName, 1, "for being the best"));
            repository.Add(new Reason(_testName, 1, "for being the best"));
            repository.Add(new Reason(_testName, 1, "for being the best"));
            var result = repository.Get(_testName, 3);

            Assert.NotNull(result);
            Assert.Equal(3, result.Count());
        }

        [Fact]
        public void GivenNumberOfResultsToReturn_WhenNumberOfResultsIsTooGreat_ShouldReturnAllResults()
        {
            var repository = CreateRepository();
            
            repository.Add(new Reason(_testName, 1, "for being the best"));
            repository.Add(new Reason(_testName, 1, "for being the best"));
            repository.Add(new Reason(_testName, 1, "for being the best"));
            var result = repository.Get(_testName, 10);

            Assert.NotNull(result);
            Assert.Equal(3, result.Count());
        }

        private ReasonRepository CreateRepository()
        {
            InitContext();
            return new ReasonRepository(_context);
        }
    }
}