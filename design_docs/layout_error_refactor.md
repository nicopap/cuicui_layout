# Refactoring error handling in layout algo

**Problem**: We need to pass around a query to get the entity name over all
size calculations. It's freaking annoying and makes the code inscrutable.

We should, instead, have a `Vec<LayoutEntityError>` for error aggregation, and just push
to it whenever we encounter an error.