using System.ComponentModel.DataAnnotations;
using System.ComponentModel.DataAnnotations.Schema;

namespace TermiBot.Karma.Models
{
    public class Entry
    {
        [Key]
        public string IdName { get; set; }
        public string DisplayName { get; set; }
        public int Karma { get; set; }

        public Entry(string displayName, int karma)
        {
            IdName = displayName.ToLower();
            DisplayName = displayName;
            Karma = karma;
        }
    }
}