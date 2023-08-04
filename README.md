# middleclick.wtf

This site converts a middle-click on a Discord image embed to a redirect to any URL you want. You can try it out [here](https://middleclick.wtf/).

## How it works

Apache checks the UserAgent. If it's Discord, it redirects to the image. If not, it redirects to the URL specified in the `redirect` query parameter.

## Why?

This was created simply out of curiosity. I wanted to see if it was possible to do this, and it turns out it is. I'll keep it online as long as it's appropriately used.
