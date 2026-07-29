# Security Policy

## Supported versions

Security updates are provided for the latest released version of pathbin.
Older releases and unreleased development snapshots are not supported.

| Version | Supported |
| --- | --- |
| Latest release | Yes |
| Older releases | No |

Users should update to the latest release before reporting a problem that may
already have been fixed.

## Reporting a vulnerability

Do not disclose suspected vulnerabilities in a public issue, discussion, pull
request, commit, or social media post.

Use the repository's **Security** tab and select **Report a vulnerability** to
send a private report:

<https://github.com/n0ta26/pathbin/security/advisories/new>

If private vulnerability reporting is unavailable, open a public issue titled
`Security contact request` without including any vulnerability details. A
maintainer will arrange a private communication channel.

Private vulnerability reporting is available only when supported by the
repository's current visibility and GitHub plan. Maintainers must verify that
the **Report a vulnerability** link above works before making the repository
public or announcing a release. The public contact-request fallback remains
available if GitHub does not provide the private reporting form.

Include as much of the following information as possible in the private
report:

- the affected pathbin version and operating system;
- the type and potential impact of the vulnerability;
- clear reproduction steps or a minimal proof of concept;
- any conditions required for exploitation;
- suggested mitigations or fixes, if known; and
- whether the issue has been disclosed elsewhere.

Do not include real credentials, tokens, personal data, or other unnecessary
sensitive information. Use test data in reproductions.

## What to expect

Maintainers will aim to:

- acknowledge a report within seven days;
- confirm whether the issue is reproducible and in scope;
- provide progress updates when practical; and
- coordinate a fix and release before public disclosure.

Response and remediation times depend on the issue's complexity and impact.
Please allow a reasonable time for investigation and remediation, and
coordinate public disclosure with the maintainers.

If a report is not considered a security vulnerability, maintainers may
redirect it to the normal issue tracker.

## Scope

Security issues may include vulnerabilities in pathbin's source code, release
artifacts, or documented installation process. Vulnerabilities in third-party
dependencies should also be reported to the affected upstream project when
appropriate.

General bugs, feature requests, and support questions that do not have a
security impact should be reported through the public issue tracker.

## Repository security controls

Maintainers keep dependency vulnerability alerts and automated security fixes
enabled in the repository settings. Dependabot checks both Cargo dependencies
and pinned GitHub Actions each week using `.github/dependabot.yml`.

GitHub availability for secret scanning, push protection, and private
vulnerability reporting depends on repository visibility and plan. Before a
visibility change or public announcement, maintainers must enable every
available control and verify the security-reporting link above. Until secret
scanning is available, maintainers should continue to scan the full Git history
with a dedicated secret scanner before each public release.
