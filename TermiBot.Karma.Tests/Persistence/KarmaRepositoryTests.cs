using System.Linq;
using TermiBot.Karma.Persistence;
using Xunit;

namespace TermiBot.Karma.Tests.Persistence
{
    public class KarmaRepositoryTests : BaseKarmaContextTests
    {
        private string _testName = "testName";
        private string _testNameWeirdlyCapitalised = "TeSTnAmE";

        private string[] _testNames =
        {
            "testName1",
            "testName2",
            "testName3",
            "testName4",
            "testName5",
            "testName6",
            "testName7",
            "testName8",
            "testName9",
            "testName10",
            "testName11",
        };

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

        [Fact]
        public void GivenLowerNumberAddedFirst_WhenGettingTopEntries_ShouldOrderByDescendingValue()
        {
            KarmaRepository repository = CreateRepository();
            
            repository.UpdateOrAdd(_testNames[1], 0);
            repository.UpdateOrAdd(_testNames[0], 999);
            var result = repository.GetTop(2).ToList();
            
            Assert.Equal(999, result[0].Karma);
            Assert.Equal(0, result[1].Karma);
        }

        [Fact]
        public void GivenAGreaterNumberParameterThanEntriesExist_WhenGettingTopEntries_ShouldReturnAll()
        {
            KarmaRepository repository = CreateRepository();
            
            repository.UpdateOrAdd(_testNames[1], 0);
            repository.UpdateOrAdd(_testNames[0], 999);
            var result = repository.GetTop(200).Count();
            
            Assert.Equal(2, result);
        }

        [Fact]
        public void WhenGettingTopEntries_ShouldReturnNumberRequested()
        {
            KarmaRepository repository = CreateRepository();
            
            repository.UpdateOrAdd(_testNames[0], 0);
            repository.UpdateOrAdd(_testNames[1], 999);
            repository.UpdateOrAdd(_testNames[2], 999);
            repository.UpdateOrAdd(_testNames[3], 999);
            repository.UpdateOrAdd(_testNames[4], 999);
            repository.UpdateOrAdd(_testNames[5], 999);
            repository.UpdateOrAdd(_testNames[6], 999);
            repository.UpdateOrAdd(_testNames[7], 999);
            repository.UpdateOrAdd(_testNames[8], 999);
            repository.UpdateOrAdd(_testNames[9], 999);
            repository.UpdateOrAdd(_testNames[10], 999);
            
            var result = repository.GetTop(2).Count();
            
            Assert.Equal(2, result);
        }

        [Fact]
        public void GivenNullInput_WhenGettingTopEntries_ShouldReturnAll()
        {
            KarmaRepository repository = CreateRepository();
            
            repository.UpdateOrAdd(_testNames[0], 0);
            repository.UpdateOrAdd(_testNames[1], 999);
            repository.UpdateOrAdd(_testNames[2], 999);
            repository.UpdateOrAdd(_testNames[3], 999);
            repository.UpdateOrAdd(_testNames[4], 999);
            var result = repository.GetTop(null).Count();
            
            Assert.Equal(5, result);
        }
        
        private KarmaRepository CreateRepository()
        {
            InitContext();
            return new KarmaRepository(_context);
        }
    }
}