using System;
using System.Collections.Generic;
using System.Linq;
using Flurl.Util;
using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Internal;
using TermiBot.Karma.Models;

namespace TermiBot.Karma.Persistence
{
    public class KarmaRepository
    {
        private readonly KarmaContext _context;
        private readonly Dictionary<string, Entry> _entries;

        public KarmaRepository(KarmaContext context)
        {
            _context = context;
            _entries = new Dictionary<string, Entry>();
        }

        public void UpdateOrAdd(string name, int karma)
        {
            string idName = name.ToLower();
            if (Exists(name))
            {
                RefreshLocalEntriesFor(idName);
                _entries[idName].Karma = karma;
                _context.Update(_entries[idName]);
            }
            else
            {
                _entries[idName] = new Entry(name, karma);
                _context.Add(_entries[idName]);
            }
            _context.SaveChanges();
        }

        public int KarmaFor(string name)
        {
            var idName = name.ToLower();
            return !Exists(idName) ? 0 : _context.Entries.First(x => x.IdName.Equals(idName)).Karma;
        }

        public bool Exists(string name)
        {
            var idName = name.ToLower();
            RefreshLocalEntriesFor(idName);
            return _entries.ContainsKey(idName);
        }

        public IEnumerable<Entry> GetTop(int? n)
        {
            var result = _context.Entries.OrderByDescending(x => x.Karma);
            return n.HasValue ? result.Take(n.Value) : result;
        }
        
        private void RefreshLocalEntriesFor(string name)
        {
            var idName = name.ToLower();
            if (!_entries.ContainsKey(idName) && _context.Entries.Any(x => x.IdName == idName))
            {
                _entries[idName] = _context.Entries.First(x => x.IdName == idName);
            }
        }
    }
}