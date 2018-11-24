using System;
using System.Collections.Generic;
using Microsoft.EntityFrameworkCore;
using Noobot.Core.Plugins;
using TermiBot.Karma.Models;
using TermiBot.Karma.Persistence;

namespace TermiBot.Karma.Plugins
{
    public class KarmaPlugin : IPlugin
    {
        private KarmaRepository _karmaRepository;
        private ReasonRepository _reasonRepository;
        private KarmaContext _context;
        
        public void Start()
        {
            Console.WriteLine("KarmaPlugin started");
            _context = new KarmaContext();
            _context.Database.Migrate();
            _karmaRepository = new KarmaRepository(_context);
            _reasonRepository = new ReasonRepository(_context);
        }

        public void Stop()
        {
            _context.Dispose();
        }

        public void Update(ChangeRequest request)
        {
            var newKarma = _karmaRepository.KarmaFor(request.Name) + request.Amount;
            _karmaRepository.UpdateOrAdd(request.Name, newKarma);
        }

        public int Get(string name)
        {
            return _karmaRepository.KarmaFor(name);
        }
    }
}