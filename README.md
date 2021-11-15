# Statistics Collector

This is the backend of a usage statistics system for Neos VR.

## How it works

1. A tracked item is spawned in-game
2. Logix on the item gathers various data (item name, item version, etc) and generates a URL for a [tracking pixel](https://en.wikipedia.org/wiki/Web_beacon)
3. The tracking pixel is loaded, sending a request to the statistics-collector server
4. The statistics-collector server logs the request to a PostgreSQL database

## What data is collected
| field                 | description | why it's collected |
| --------------------- | ----------- | ------------------ |
| timestamp             | Time the item was spawned | core usage statistics data |
| item_name             | Slot name of the spawned item | core usage statistics data |
| item_id               | Internal id of the item (intended to stay static even if the slot name changes) | core usage statistics data |
| neos_version          | Neos client version of whoever spawned the item | Tracking Neos bugs that affect statistics collection |
| session_id            | Neos session id the item was spawned into | removing spam caused by leaving my items in worlds |
| world_url             | Neos world url the item was spawned into | removing spam caused by leaving my items in worlds |
| protocol_version      | statistics-collector API version | tracking when I can remove support for old API endpoints |
| client_major_version  | statistics LogiX version | tracking rollout of bugfixes in the client LogiX |
| client_minor_version  | statistics LogiX version | tracking rollout of bugfixes in the client LogiX |
| cache_nonce           | Used for cache busting. | Recorded for debugging reasons |

## The output
After filtering out various types of bad data, these are the results that I actually look at:

item | usage percent
--- | ---
EyeFinder | 15.5072463768115942
adDragon | 15.3623188405797101
ASMR Brush | 14.7101449275362319
ad2dGirl | 14.7101449275362319
ad3dGirl | 14.3478260869565217
Colliderless Gun | 7.9710144927536232
👺Tip | 2.8985507246376812
MathGun | 2.7536231884057971
The chosen perspective | 2.7536231884057971
BootySetterTip | 2.3913043478260870
Feetus Deletus | 2.3188405797101449
YeetTip | 1.1594202898550725
Shitty Laser Pointer | 0.86956521739130434783
Colliderizer | 0.65217391304347826087
Hammer of Deactivation | 0.43478260869565217391
ColliderFlashTip | 0.36231884057971014493
AllocatingUserTip | 0.36231884057971014493
Performance Graph | 0.21739130434782608696
PersistentSetterTip | 0.14492753623188405797
BullFody | 0.07246376811594202899

Things that surprised me:
- People use my stupid meme advertisments way more than I expected
- People use my performance graph way less than I expected :(

## Changelog
[Full changelog here](doc/changelog.md)

## Frequently Asked Questions

### Who are you?
I'm `runtime` in-game. [`U-runtime`](https://api.neos.com/api/users/U-runtime) if you need my user id. I make tools and stuff. If you want to talk to me about this hit me up in-game, on discord, via email, whatever.

### Why are you doing this?
I recieve almost no in-game feedback about my creations. The idea is that by gathering usage statistics for the various tools I maintain I can better direct my efforts towards the tools people are actually using.

### This makes me uncomfortable
That's not a question, but sorry I guess? You realize that pretty much anything you touch on the internet is going to have some form of usage statistics?

### Isn't this abusing an exploit?
No. As per Frooxius in [Neos Issue #883](https://github.com/Neos-Metaverse/NeosPublic/issues/883) this is not a bug: it is working as intended. Also, this behavior has been known for *more than a year*, so it's not as if I'm sneaking this in before the ramifications have had time to be fully understood.

### Isn't this against Neos's privacy policy?
No. [Neos's Privacy Policy](https://wiki.neos.com/Neos_Wiki:Privacy_policy#Community_Content) shrugs community content privacy issues off onto the content creators. So here's a [generic copy/pasted privacy policy](privacy_policy.html) for this application. Enjoy.

### Could this be used to join hidden sessions?
Yes, as long as the joiner meets the other requirments. You can't bypass Contacts+/Contacts/Private just becuase you have the session ID. If you need *real* security and not just [security by obscurity](https://en.wikipedia.org/wiki/Security_through_obscurity) don't use the the Anyone or Registered User access levels.

### Are you using this to join hidden sessions?
No. Why would I want to do that? I don't enjoy being around people who don't want me there, so sneaking into sessions uninvited would be a colossal waste of my time.

If someone *does* join your hidden session they'll show up in your Neos logs. Neos does not delete old logs so you can go read your *entire* log history to see if someone's been sneaking around. If someone is entering your hidden sessions and harrasing you I suggest you open a [moderation ticket](https://moderation.neos.com/).

Finally, if you encounter somone who claims runtime is joining hidden sessions uninvited, please ask them to provide proof. I guarantee they will be unable to. I would love it if people stop slandering me.
