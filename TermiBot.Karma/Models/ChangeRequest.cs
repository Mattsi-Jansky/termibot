namespace TermiBot.Karma.Models
{
    public class ChangeRequest
    {
        public string Name { get; }
        public int Amount { get; }

        public ChangeRequest(string name, int amount)
        {
            Name = name;
            Amount = amount;
        }
    }
}