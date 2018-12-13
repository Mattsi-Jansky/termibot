using System;
using System.Collections.Generic;
using Microsoft.EntityFrameworkCore;
using Noobot.Core.Plugins;
using TermiBot.Karma.Models;
using TermiBot.Karma.Persistence;

namespace TermiBot.Karma.Plugins
{
    public class KarmaRepositoryPlugin : IPlugin
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
            if (!String.IsNullOrEmpty(request.Reason)) AddReason(request);
        }

        private void AddReason(ChangeRequest request)
        {
            _reasonRepository.Add(Reason.FromChangeRequest(request));
        }

        public int Get(string name)
        {
            return _karmaRepository.KarmaFor(name);
        }

        public IEnumerable<Entry> GetTop(int? n)
        {
            return _karmaRepository.GetTop(n);
        }
    }
}