/// Parses out Windows permissions.
#[cfg(target_family = "windows")]
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct WindowsPermissions {
    /// The permission bits.
    pub bits: u32,
}

#[cfg(target_family = "windows")]
impl WindowsPermissions {
    /// Returns whether the given flag is set in the inner permission bits.
    #[inline]
    const fn has_flag(self, flag: u32) -> bool {
        self.bits & flag != 0
    }

    /// Returns whether this [`WindowsPermissions`] is read-only.
    #[must_use]
    pub const fn is_readonly(&self) -> bool {
        self.has_flag(1 << 0)
    }

    /// Returns whether this [`WindowsPermissions`] is hidden.
    #[must_use]
    pub const fn is_hidden(&self) -> bool {
        self.has_flag(1 << 1)
    }

    /// Returns whether this [`WindowsPermissions`] is a system entry.
    #[must_use]
    pub const fn is_system(&self) -> bool {
        self.has_flag(1 << 2)
    }

    /// Returns whether this [`WindowsPermissions`] is an archive.
    #[must_use]
    pub const fn is_archive(&self) -> bool {
        self.has_flag(1 << 4)
    }
}

#[cfg(target_family = "windows")]
impl std::fmt::Display for WindowsPermissions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[inline]
        fn write(f: &mut std::fmt::Formatter<'_>, b: bool, c: char) -> std::fmt::Result {
            write!(f, "{}", if b { c } else { '-' })
        }

        write(f, self.is_readonly(), 'r')?;
        write(f, self.is_archive(), 'a')?;
        write(f, self.is_hidden(), 'h')?;
        write(f, self.is_system(), 's')?;

        Ok(())
    }
}
