use core::fmt;

use crate::{Component, LanguageTag, Part, ValueString};

/// A CPE 2.3 attribute name.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Attribute {
    /// Product class.
    Part,
    /// Product creator or manufacturer.
    Vendor,
    /// Product name.
    Product,
    /// Product release version.
    Version,
    /// Product update, service pack, or point release.
    Update,
    /// Deprecated legacy edition.
    Edition,
    /// User-interface language.
    Language,
    /// Market or end-user software edition.
    SoftwareEdition,
    /// Target software environment.
    TargetSoftware,
    /// Target instruction-set architecture.
    TargetHardware,
    /// Other product-specific information.
    Other,
}

impl fmt::Display for Attribute {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Part => "part",
            Self::Vendor => "vendor",
            Self::Product => "product",
            Self::Version => "version",
            Self::Update => "update",
            Self::Edition => "edition",
            Self::Language => "language",
            Self::SoftwareEdition => "sw_edition",
            Self::TargetSoftware => "target_sw",
            Self::TargetHardware => "target_hw",
            Self::Other => "other",
        })
    }
}

/// An owned CPE 2.3 name representable by the formatted-string binding.
///
/// Parsing and [`fmt::Display`] use the formatted-string binding. Structural
/// equality compares the semantic fields; CPE name matching is a separate
/// operation and is not implemented by `Eq`. In particular, the language
/// component follows Figure 6-3's narrower binding grammar rather than every
/// language tag allowed by the abstract WFN model.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Cpe {
    pub(crate) part: Component<Part>,
    pub(crate) vendor: Component<ValueString>,
    pub(crate) product: Component<ValueString>,
    pub(crate) version: Component<ValueString>,
    pub(crate) update: Component<ValueString>,
    pub(crate) edition: Component<ValueString>,
    pub(crate) language: Component<LanguageTag>,
    pub(crate) software_edition: Component<ValueString>,
    pub(crate) target_software: Component<ValueString>,
    pub(crate) target_hardware: Component<ValueString>,
    pub(crate) other: Component<ValueString>,
}

impl Cpe {
    /// Creates a builder whose unspecified attributes default to logical `ANY`.
    #[must_use]
    pub fn builder(part: impl Into<Component<Part>>) -> CpeBuilder {
        CpeBuilder {
            cpe: Self {
                part: part.into(),
                vendor: Component::Any,
                product: Component::Any,
                version: Component::Any,
                update: Component::Any,
                edition: Component::Any,
                language: Component::Any,
                software_edition: Component::Any,
                target_software: Component::Any,
                target_hardware: Component::Any,
                other: Component::Any,
            },
        }
    }

    /// Returns the product class component.
    #[must_use]
    pub const fn part(&self) -> &Component<Part> {
        &self.part
    }

    /// Returns the vendor component.
    #[must_use]
    pub const fn vendor(&self) -> &Component<ValueString> {
        &self.vendor
    }

    /// Returns the product component.
    #[must_use]
    pub const fn product(&self) -> &Component<ValueString> {
        &self.product
    }

    /// Returns the version component.
    #[must_use]
    pub const fn version(&self) -> &Component<ValueString> {
        &self.version
    }

    /// Returns the update component.
    #[must_use]
    pub const fn update(&self) -> &Component<ValueString> {
        &self.update
    }

    /// Returns the deprecated legacy-edition component.
    #[must_use]
    pub const fn edition(&self) -> &Component<ValueString> {
        &self.edition
    }

    /// Returns the language component.
    #[must_use]
    pub const fn language(&self) -> &Component<LanguageTag> {
        &self.language
    }

    /// Returns the software-edition component.
    #[must_use]
    pub const fn software_edition(&self) -> &Component<ValueString> {
        &self.software_edition
    }

    /// Returns the target-software component.
    #[must_use]
    pub const fn target_software(&self) -> &Component<ValueString> {
        &self.target_software
    }

    /// Returns the target-hardware component.
    #[must_use]
    pub const fn target_hardware(&self) -> &Component<ValueString> {
        &self.target_hardware
    }

    /// Returns the residual `other` component.
    #[must_use]
    pub const fn other(&self) -> &Component<ValueString> {
        &self.other
    }
}

impl Default for Cpe {
    fn default() -> Self {
        Self::builder(Component::Any).build()
    }
}

impl fmt::Display for Cpe {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "cpe:2.3:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
            self.part,
            self.vendor,
            self.product,
            self.version,
            self.update,
            self.edition,
            self.language,
            self.software_edition,
            self.target_software,
            self.target_hardware,
            self.other
        )
    }
}

/// A builder for an owned [`Cpe`] name.
#[derive(Clone, Debug)]
pub struct CpeBuilder {
    cpe: Cpe,
}

impl CpeBuilder {
    /// Sets the vendor component.
    #[must_use]
    pub fn vendor(mut self, value: impl Into<Component<ValueString>>) -> Self {
        self.cpe.vendor = value.into();
        self
    }

    /// Sets the product component.
    #[must_use]
    pub fn product(mut self, value: impl Into<Component<ValueString>>) -> Self {
        self.cpe.product = value.into();
        self
    }

    /// Sets the version component.
    #[must_use]
    pub fn version(mut self, value: impl Into<Component<ValueString>>) -> Self {
        self.cpe.version = value.into();
        self
    }

    /// Sets the update component.
    #[must_use]
    pub fn update(mut self, value: impl Into<Component<ValueString>>) -> Self {
        self.cpe.update = value.into();
        self
    }

    /// Sets the deprecated legacy-edition component.
    #[must_use]
    pub fn edition(mut self, value: impl Into<Component<ValueString>>) -> Self {
        self.cpe.edition = value.into();
        self
    }

    /// Sets the language component.
    #[must_use]
    pub fn language(mut self, value: impl Into<Component<LanguageTag>>) -> Self {
        self.cpe.language = value.into();
        self
    }

    /// Sets the software-edition component.
    #[must_use]
    pub fn software_edition(mut self, value: impl Into<Component<ValueString>>) -> Self {
        self.cpe.software_edition = value.into();
        self
    }

    /// Sets the target-software component.
    #[must_use]
    pub fn target_software(mut self, value: impl Into<Component<ValueString>>) -> Self {
        self.cpe.target_software = value.into();
        self
    }

    /// Sets the target-hardware component.
    #[must_use]
    pub fn target_hardware(mut self, value: impl Into<Component<ValueString>>) -> Self {
        self.cpe.target_hardware = value.into();
        self
    }

    /// Sets the residual `other` component.
    #[must_use]
    pub fn other(mut self, value: impl Into<Component<ValueString>>) -> Self {
        self.cpe.other = value.into();
        self
    }

    /// Finishes the name.
    #[must_use]
    pub fn build(self) -> Cpe {
        self.cpe
    }
}
