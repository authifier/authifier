![rAuth](./banner.png)

Internally rAuth stores emails with and without special characters, `+.`.
- This means we can support plus signing without allowing the same email to sign up multiple times.
  - For example, `inbox+a@example.com` and `inbox+b@example.com` are treated as equal.
  - But since we are still storing the original email, we still send them marked with the user's sign.
- In the case of Gmail, all emails with dots are forwarded to those without them, this can lead to some [unfortunate situations](https://jameshfisher.com/2018/04/07/the-dots-do-matter-how-to-scam-a-gmail-user/).
  - Generally, we treat all emails with dots as their non-dot counterpart when checking if an email exists.
  - This may inconvenience some users but I would rather avoid situations like above or duplicate accounts.
- When logging in, the email given is checked against the original email and nothing else.
  
