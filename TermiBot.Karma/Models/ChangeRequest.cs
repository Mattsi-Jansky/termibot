using Flurl.Http;

namespace TermiBot.Karma.Models
{
    public class ChangeRequest
    {
        public string Name { get; }
        public int Amount { get; }
        public string Reason { get; }

        public ChangeRequest(string name, int amount)
        {
            Name = name;
            Amount = amount;
        }

        public ChangeRequest(ChangeRequest request, string reason)
        {
            Name = request.Name;
            Amount = request.Amount;
            Reason = reason;
        }

        public ChangeRequest(string name, int amount, string reason)
        {
            Name = name;
            Amount = amount;
            Reason = reason;
        }
    }
}