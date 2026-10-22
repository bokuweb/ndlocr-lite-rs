# Single-column fallback Page regression

Known-font synthetic FVB development image `fvb_vertical_arial_36_121_pad2`, frozen before
its initial NN comparison. The PNG and literal GT are copied byte-for-byte from that corpus.
The GT contains 121 Unicode scalars, including U+3000. The expected raw Page output is
120 scalars; its missing U+3000 remains a measured character error. Both bytes and the
nonwhitespace body are asserted separately. This is not an independent-document result.

See `docs/vertical_page_supported_page_regression.md` for Red/Green and reuse requirements.
Official DEIM, PARSeq30, and charset assets are mandatory when executing the ignored test.
