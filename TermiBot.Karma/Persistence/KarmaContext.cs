using Microsoft.EntityFrameworkCore;
using TermiBot.Karma.Models;

namespace TermiBot.Karma.Persistence
{
    public class KarmaContext : DbContext
    {
        public DbSet<Entry> Entries { get; set; }
        public DbSet<Reason> Reasons { get; set; }
        private string _dataSourceFilename;
        private const string DefaultDataSourceFilename = "karma.db";

        public KarmaContext()
        {
            _dataSourceFilename = DefaultDataSourceFilename;
        }

        public KarmaContext(string dataSourceFilename)
        {
            _dataSourceFilename = dataSourceFilename;
        }

        protected override void OnConfiguring(DbContextOptionsBuilder optionsBuilder)
        {
            optionsBuilder.UseSqlite($"Data Source={_dataSourceFilename}");
        }
    }
}