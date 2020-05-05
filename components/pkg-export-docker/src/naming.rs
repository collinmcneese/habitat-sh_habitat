use crate::{error::Result,
            RegistryType};
use clap::ArgMatches;
use failure::SyncFailure;
use habitat_core::{package::{FullyQualifiedPackageIdent,
                             Identifiable},
                   ChannelIdent};
use handlebars::Handlebars;
use serde::Serialize;

const DEFAULT_IMAGE_NAME_TEMPLATE: &str = "{{pkg_origin}}/{{pkg_name}}";
const VERSION_RELEASE_TAG_TEMPLATE: &str = "{{pkg_version}}-{{pkg_release}}";
const VERSION_TAG_TEMPLATE: &str = "{{pkg_version}}";

/// Helper macro to mark all Handlebars templating calls that are not based
/// on user input, and can thus be safely unwrapped.
macro_rules! safe {
    ($render_result:expr) => {
        $render_result.expect("We are in control of all inputs to this rendering, and thus the \
                           result should always be safe to unwrap")
    };
}

/// An image naming policy.
///
/// This is a value struct which captures the naming and tagging intentions for an image.
#[derive(Debug, Default)]
pub struct Naming {
    /// An optional custom image name which would override a computed default value.
    custom_image_name_template:  Option<String>,
    /// Whether or not to tag the image with a latest value.
    include_latest_tag:          bool,
    /// Whether or not to tag the image with a value containing a version from a Package
    /// Identifier.
    include_version_tag:         bool,
    /// Whether or not to tag the image with a value containing a version and release from a
    /// Package Identifier.
    include_version_release_tag: bool,
    /// An optional custom tag value for the image.
    custom_tag_template:         Option<String>,

    // TODO (CM): I don't think either of these really belongs to this
    // Naming type

    // TODO (CM): really, url and type are conceptually part of a
    // single new type.
    /// A URL to a custom Docker registry to publish to. This will be used as part of every tag
    /// before pushing.
    pub registry_url:  Option<String>, // TODO (CM): Option<Url>
    /// The type of registry we're publishing to. Ex: Amazon, Docker, Google, Azure.
    pub registry_type: RegistryType,
}

impl From<&ArgMatches<'_>> for Naming {
    fn from(matches: &ArgMatches) -> Self {
        let registry_type =
            clap::value_t!(matches.value_of("REGISTRY_TYPE"), RegistryType).unwrap_or_default();

        // TODO (CM): If registry_type is Docker, we must set this to
        // dockerhub. Otherwise, it MUST be present, because of how
        // clap is set up.
        let registry_url = matches.value_of("REGISTRY_URL").map(ToString::to_string);

        Naming { custom_image_name_template: matches.value_of("IMAGE_NAME")
                                                    .map(ToString::to_string),
                 include_latest_tag: !matches.is_present("NO_TAG_LATEST"),
                 include_version_tag: !matches.is_present("NO_TAG_VERSION"),
                 include_version_release_tag: !matches.is_present("NO_TAG_VERSION_RELEASE"),
                 custom_tag_template: matches.value_of("TAG_CUSTOM").map(ToString::to_string),
                 registry_url,
                 registry_type }
    }
}

// TODO (CM): must validate custom names and tags w/r/t tag spec
// https://docs.docker.com/engine/reference/commandline/tag/
//
// An image name is made up of slash-separated name components,
// optionally prefixed by a registry hostname. The hostname must
// comply with standard DNS rules, but may not contain underscores. If
// a hostname is present, it may optionally be followed by a port
// number in the format :8080. If not present, the command uses
// Docker’s public registry located at registry-1.docker.io by
// default. Name components may contain lowercase letters, digits and
// separators. A separator is defined as a period, one or two
// underscores, or one or more dashes. A name component may not start
// or end with a separator.
//
// A tag name must be valid ASCII and may contain lowercase and
// uppercase letters, digits, underscores, periods and dashes. A tag
// name may not start with a period or a dash and may contain a
// maximum of 128 characters.

impl Naming {
    // TODO (CM): I am skeptical of use of "channel" in any container
    // identifier, since that is not anything inherent to the package
    // we are containerizing.

    /// Return the image name, along with a (possibly empty) vector of
    /// additional tags.
    pub fn image_identifiers(&self,
                             ident: &FullyQualifiedPackageIdent,
                             channel: &ChannelIdent)
                             -> Result<(String, Vec<String>)> {
        let context = Self::rendering_context(ident, channel);

        let name = self.image_name(&context)?;
        let tags = vec![self.latest_tag(),
                        self.version_tag(&context),
                        self.version_release_tag(&context),
                        self.custom_tag(&context)?].into_iter()
                                                   .filter_map(|e| e)
                                                   .collect();
        Ok((name, tags))
    }

    ////////////////////////////////////////////////////////////////////////

    fn image_name<S>(&self, context: &S) -> Result<String>
        where S: Serialize
    {
        let image_name = if let Some(ref template) = self.custom_image_name_template {
            Self::render(&template, &context)?
        } else {
            safe!(Self::render(DEFAULT_IMAGE_NAME_TEMPLATE, &context))
        };

        // TODO (CM): perhaps we should prepend the registry URL to
        // tags only if we going to push (and at the time we are
        // pushing, not here)
        let image_name = match self.registry_url {
            Some(ref url) => format!("{}/{}", url, image_name),
            None => image_name,
        };

        Ok(image_name.to_lowercase())
    }

    fn version_release_tag<S>(&self, context: &S) -> Option<String>
        where S: Serialize
    {
        if self.include_version_release_tag {
            Some(safe!(Self::render(VERSION_RELEASE_TAG_TEMPLATE, &context)))
        } else {
            None
        }
    }

    fn version_tag<S>(&self, context: &S) -> Option<String>
        where S: Serialize
    {
        if self.include_version_tag {
            Some(safe!(Self::render(VERSION_TAG_TEMPLATE, &context)))
        } else {
            None
        }
    }

    // TODO (CM): not sure how useful this is, since I think "latest"
    // is *always* created.
    fn latest_tag(&self) -> Option<String> {
        if self.include_latest_tag {
            Some("latest".to_string())
        } else {
            None
        }
    }

    fn custom_tag<S>(&self, context: &S) -> Result<Option<String>>
        where S: Serialize
    {
        if let Some(ref custom_tag_template) = self.custom_tag_template {
            Ok(Some(Self::render(custom_tag_template, &context)?))
        } else {
            Ok(None)
        }
    }

    // TODO (CM): need to generate better error cases for this... if a
    // user inputs invalid input, the results can be cryptic:
    //
    // For instance, a template of "{{" give the error
    //
    //   ✗✗✗ Template "Unnamed template" line 1, col 3: invalid
    //   handlebars syntax.
    //
    // Not terribly useful, as there's no indication of what the
    // offending input is.
    //
    // We might want to pass more context to this render call (so
    // users can know which template was the offender)
    fn render<S>(template: &str, context: &S) -> Result<String>
        where S: Serialize
    {
        Ok(Handlebars::new().template_render(template, context)
                            .map_err(SyncFailure::new)
                            .map(|s| s.to_lowercase())?)
    }

    fn rendering_context(ident: &FullyQualifiedPackageIdent,
                         channel: &ChannelIdent)
                         -> impl Serialize {
        json!({
            "pkg_origin": ident.origin(),
            "pkg_name": ident.name(),
            "pkg_version": ident.version(),
            "pkg_release": ident.release(),
            "channel": channel,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ident() -> FullyQualifiedPackageIdent { "core/foo/1.2.3/20200430153200".parse().unwrap() }

    fn context() -> impl Serialize {
        let ident = ident();
        let channel = ChannelIdent::default();
        Naming::rendering_context(&ident, &channel)
    }

    #[test]
    fn default_naming_policy() {
        let naming = Naming::default();
        let context = context();

        assert!(naming.latest_tag().is_none());
        assert!(naming.custom_tag(&context).unwrap().is_none());
        assert!(naming.version_tag(&context).is_none());
        assert!(naming.version_release_tag(&context).is_none());

        assert_eq!(naming.image_name(&context).unwrap(), "core/foo");
    }

    #[test]
    fn latest_tag() {
        let mut naming = Naming::default();
        naming.include_latest_tag = true;
        assert_eq!(naming.latest_tag().unwrap(), "latest");
    }

    #[test]
    fn version_tag() {
        let mut naming = Naming::default();
        naming.include_version_tag = true;

        let context = context();
        assert_eq!(naming.version_tag(&context).unwrap(), "1.2.3");
    }

    #[test]
    fn version_release_tag() {
        let mut naming = Naming::default();
        naming.include_version_release_tag = true;

        let context = context();
        assert_eq!(naming.version_release_tag(&context).unwrap(),
                   "1.2.3-20200430153200");
    }

    #[test]
    fn image_name_with_registry_url() {
        let mut naming = Naming::default();
        // TODO (CM): IMPLEMENTATION QUIRK
        // Registry type has no bearing on this! Fix it!
        naming.registry_url = Some(String::from("registry.mycompany.com:8080/v1"));

        let context = context();

        let name = naming.image_name(&context).unwrap();
        assert_eq!(name, "registry.mycompany.com:8080/v1/core/foo");
    }

    #[test]
    fn custom_image_names() {
        let context = context();

        // Template, Expected Result
        //
        // A "Some" expected result is something you expect to successfully
        // pass. A "None", on the other hand, is something you expect
        // to throw an error.
        let inputs = vec![

            // Valid inputs
            ("monkeys", Some("monkeys")),
            ("{{pkg_name}}", Some("foo")),
            ("{{pkg_origin}}-{{pkg_name}}", Some("core-foo")),
            ("{{pkg_origin}}-{{pkg_name}}-{{pkg_version}}", Some("core-foo-1.2.3")),
            ("{{pkg_origin}}-{{pkg_name}}-{{pkg_version}}-{{pkg_release}}",
             Some("core-foo-1.2.3-20200430153200")),
            ("{{pkg_origin}}-{{pkg_name}}-{{pkg_version}}-{{pkg_release}}-{{channel}}",
             Some("core-foo-1.2.3-20200430153200-stable")),
            ("super-{{pkg_origin}}-wacky-{{pkg_name}}-funtime-{{pkg_version}}-container-{{pkg_release}}-party-{{channel}}-ohemgee",
             Some("super-core-wacky-foo-funtime-1.2.3-container-20200430153200-party-stable-ohemgee")),

            // Invalid inputs
            ("{{" , None),
            // ("{{not_a_valid_variable}}-{{pkg_name}}", None),
            // ("", None),
            // more examples of things that violate the tagging spec
        ];

        for (template, expected) in inputs {
            let mut naming = Naming::default();

            let template = String::from(template);
            naming.custom_image_name_template = Some(template.clone());
            let actual_name = naming.image_name(&context);

            if let Some(expected_name) = expected {
                // expected success
                assert!(actual_name.is_ok());
                let actual_name = actual_name.unwrap();
                assert_eq!(actual_name, expected_name,
                           "Expected template '{}' to generate image name '{}', but it generated \
                            '{}'",
                           template, expected_name, actual_name);
            } else {
                // expected failure
                if let Err(actual_name) = actual_name {
                    assert!(false,
                            "Expected template '{}' to fail to generate an image name, but it \
                             generated '{}'",
                            template, actual_name);
                }
            }
        }
    }

    // TODO (CM): there is a bunch of duplication here :(

    #[test]
    fn custom_tag_names() {
        let context = context();

        // Template, Expected Result
        //
        // A "Some" expected result is something you expect to successfully
        // pass. A "None", on the other hand, is something you expect
        // to throw an error.
        let inputs = vec![

            // Valid inputs
            ("monkeys", Some("monkeys")),
            ("{{pkg_name}}", Some("foo")),
            ("{{pkg_origin}}-{{pkg_name}}", Some("core-foo")),
            ("{{pkg_origin}}-{{pkg_name}}-{{pkg_version}}", Some("core-foo-1.2.3")),
            ("{{pkg_origin}}-{{pkg_name}}-{{pkg_version}}-{{pkg_release}}",
             Some("core-foo-1.2.3-20200430153200")),
            ("{{pkg_origin}}-{{pkg_name}}-{{pkg_version}}-{{pkg_release}}-{{channel}}",
             Some("core-foo-1.2.3-20200430153200-stable")),
            ("super-{{pkg_origin}}-wacky-{{pkg_name}}-funtime-{{pkg_version}}-container-{{pkg_release}}-party-{{channel}}-ohemgee",
             Some("super-core-wacky-foo-funtime-1.2.3-container-20200430153200-party-stable-ohemgee")),

            // Invalid inputs
            ("{{" , None),
            // ("{{not_a_valid_variable}}-{{pkg_name}}", None),
            // ("", None),
            // more examples of things that violate the tagging spec
        ];

        for (template, expected) in inputs {
            let mut naming = Naming::default();

            let template = String::from(template);
            naming.custom_tag_template = Some(template.clone());
            let actual_tag = naming.custom_tag(&context);

            if let Some(expected_tag) = expected {
                // expected success
                assert!(actual_tag.is_ok());
                let actual_tag = actual_tag.unwrap().unwrap(); // Result<Option<String>>
                assert_eq!(actual_tag, expected_tag,
                           "Expected template '{}' to generate tag '{}', but it generated '{}'",
                           template, expected_tag, actual_tag);
            } else {
                // expected failure
                if let Err(actual_tag) = actual_tag {
                    assert!(false,
                            "Expected template '{}' to fail to generate a tag, but it generated \
                             '{}'",
                            template, actual_tag);
                }
            }
        }
    }

    #[test]
    fn default_image_identifiers() {
        let naming = Naming::default();
        let ident = ident();
        let channel = ChannelIdent::default();

        let (name, tags) = naming.image_identifiers(&ident, &channel).unwrap();

        assert_eq!(name, "core/foo");
        assert!(tags.is_empty());
    }

    #[test]
    fn all_the_image_identifiers() {
        let naming = Naming { custom_image_name_template:
                                  Some(String::from("my-nifty/{{pkg_name}}")),
                              include_latest_tag:          true,
                              include_version_tag:         true,
                              include_version_release_tag: true,
                              custom_tag_template:         Some(String::from("new-hotness")),
                              registry_url:
                                  Some(String::from("registry.mycompany.com:8080/v1")),
                              registry_type:               RegistryType::Docker, };

        let ident = ident();
        let channel = ChannelIdent::default();

        let (name, tags) = naming.image_identifiers(&ident, &channel).unwrap();

        assert_eq!(name, "registry.mycompany.com:8080/v1/my-nifty/foo");
        assert_eq!(tags,
                   ["latest", "1.2.3", "1.2.3-20200430153200", "new-hotness"]);
    }
}
