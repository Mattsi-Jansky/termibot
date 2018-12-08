using TermiBot.Karma.Persistence;
using Xunit;

namespace TermiBot.Karma.Tests.Persistence
{
    public class KarmaRepositoryTests : BaseKarmaContextTests
    {
        private string _testName = "testName";
        private string _testNameWeirdlyCapitalised = "TeSTnAmE";

        [Fact]
        public void ShouldAddEntry()
        {
            var repository = CreateRepository();
            
            repository.UpdateOrAdd(_testName, 0);
            
            Assert.True(repository.Exists(_testName));
        }

        [Fact]
        public void ShouldAddMultipleEntriesOfSameName()
        {
            KarmaRepository repository = CreateRepository();
            
            repository.UpdateOrAdd(_testName, 0);
            repository.UpdateOrAdd(_testName, 2);
            
            Assert.True(repository.Exists(_testName));
            Assert.Equal(2, repository.KarmaFor(_testName));
        }

        [Fact]
        public void ShouldReadPriorInstancesRecords()
        {
            KarmaRepository repository = CreateRepository();
            
            repository.UpdateOrAdd(_testName, 999);
            
            repository = CreateRepository();

            Assert.Equal(999, repository.KarmaFor(_testName));
        }

        [Fact]
        public void ShouldWriteToPriorInstanceRecords()
        {
            KarmaRepository repository = CreateRepository();
            
            repository.UpdateOrAdd(_testName, 0);
            
            repository = CreateRepository();
            repository.UpdateOrAdd(_testName, 2);
            
            Assert.True(repository.Exists(_testName));
            Assert.Equal(2, repository.KarmaFor(_testName));
        }

        [Fact]
        public void ShouldTreatDifferentCapitalisationsTheSame()
        {
            KarmaRepository repository = CreateRepository();
            
            repository.UpdateOrAdd(_testNameWeirdlyCapitalised, 0);
            repository.UpdateOrAdd(_testName, 999);
            
            Assert.Equal(999, repository.KarmaFor(_testNameWeirdlyCapitalised));
        }
        
        private KarmaRepository CreateRepository()
        {
            InitContext();
            return new KarmaRepository(_context);
        }
    }
}