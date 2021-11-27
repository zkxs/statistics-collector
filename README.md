# Statistics Collector

This is the backend of a usage statistics system for Neos VR.

## How it works

1. A tracked item is spawned in-game
2. Logix on the item gathers various data (item name, item version, etc) and generates a URL for a [tracking pixel](https://en.wikipedia.org/wiki/Web_beacon)
3. The tracking pixel is loaded, sending a request to the statistics-collector server
4. The statistics-collector server logs the request to a PostgreSQL database

## What data is collected
| Field                 | Description | Why it is Collected |
| --------------------- | ----------- | ------------------- |
| timestamp             | Time the item was spawned | Core usage statistics data |
| item_name             | Slot name of the spawned item | Core usage statistics data |
| item_id               | Internal id of the item (intended to stay static even if the slot name changes) | Core usage statistics data |
| neos_version          | Neos client version of whoever spawned the item | Tracking Neos bugs that affect statistics collection |
| world_url             | Neos world url the item was spawned into | Removing spam caused by saving my items in worlds |
| client_major_version  | Statistics client LogiX version | Tracking rollout of bugfixes in the client LogiX |
| client_minor_version  | Statistics client LogiX version | Tracking rollout of bugfixes in the client LogiX |
| cache_nonce           | Random number | Not recorded; used for cache busting |

## The output
After filtering out various types of bad data, the results of the statistics collector are usage percentage of my items over a rolling two-week window. Actual output from 2021-11-27:

item | uses | usage percent
---|---|---
EyeFinder | 332 | 47.8386167146974063
adDragon | 132 | 19.0201729106628242
YeetTip | 62 | 8.9337175792507205
ASMR Brush | 33 | 4.7550432276657061
BootySetterTip | 26 | 3.7463976945244957
Colliderless Gun | 25 | 3.6023054755043228
👺Tip | 23 | 3.3141210374639769
Feetus Deletus | 23 | 3.3141210374639769
MathGun | 22 | 3.1700288184438040
Performance Graph | 7 | 1.0086455331412104
The chosen perspective | 3 | 0.43227665706051873199
Shitty Laser Pointer | 2 | 0.28818443804034582133
AllocatingUserTip | 1 | 0.14409221902017291066
ColliderFlashTip | 1 | 0.14409221902017291066
ad2dGirl | 1 | 0.14409221902017291066
Hammer of Deactivation | 1 | 0.14409221902017291066

Things that surprised me:
- People use my performance graph way less than I expected. Probably not worth working on my planned improvements if no one's using it.
- People use my stupid meme advertisements way more than I expected. At least it was a high-effort shitpost.
- EyeFinder took about 10 minutes to make and is my most-used item, easily beating items I spent hours of dev time on.

## Changelog
[Full changelog here](doc/changelog.md)

## Frequently Asked Questions

### Who are you?
I'm `runtime` in-game. [`U-runtime`](https://api.neos.com/api/users/U-runtime) if you need my user id. I make tools and stuff. If you want to talk to me about this hit me up in-game, on discord, via email, whatever.

### Why are you doing this?
I receive almost no in-game feedback about my creations. The idea is that by gathering usage statistics for the various tools I maintain I can better direct my efforts towards the tools people are actually using.

### This makes me uncomfortable
That's not a question, but sorry I guess? You realize that pretty much anything you touch on the internet is going to have some form of usage statistics? As far as statistics go "item X was spawned at Y time" is pretty light.

### Isn't this abusing an exploit?
No. As per Frooxius in [Neos Issue #883](https://github.com/Neos-Metaverse/NeosPublic/issues/883) this is not a bug: it is working as intended. Also, this behavior has been known for *more than a year*, so it's not as if I'm sneaking this in before the ramifications have had time to be fully understood.

### Isn't this against Neos's privacy policy?
No. [Neos's Privacy Policy](https://wiki.neos.com/Neos_Wiki:Privacy_policy#Community_Content) shrugs community content privacy issues off onto the content creators. So here's a [generic copy/pasted privacy policy](privacy_policy.html) for this application. Enjoy.

### Could this be used to join hidden sessions?
No. The current version does not collect session IDs.
