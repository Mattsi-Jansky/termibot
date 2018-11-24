using System;
using System.Collections.Generic;
using System.Linq;
using Microsoft.EntityFrameworkCore.Internal;
using TermiBot.Karma.Models;

namespace TermiBot.Karma.Persistence
{
    public class ReasonRepository
    {
        private readonly KarmaContext _context;

        public ReasonRepository(KarmaContext context)
        {
            _context = context;
        }

        public IEnumerable<Reason> Get(string name)
        {
            return _context.Reasons.Where(x => x.Name == name);
        }

        public void Add(Reason reason)
        {
            _context.Reasons.Add(reason);
            _context.SaveChanges();
        }
    }
}