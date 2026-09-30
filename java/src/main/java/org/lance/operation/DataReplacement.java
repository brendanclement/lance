/*
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
package org.lance.operation;

import org.lance.fragment.DataFile;

import com.google.common.base.MoreObjects;

import java.util.List;
import java.util.Objects;

/**
 * Replace the values of some columns of existing fragments with new data files, one per fragment,
 * each holding every physical row of its fragment. Every new file must write the same fields. This
 * populates an all-NULL column or recomputes a stored one without rewriting the fragment's other
 * columns.
 *
 * <p>For each fragment, the new file:
 *
 * <ul>
 *   <li>swaps in place for an existing file that writes exactly the same fields in the same file
 *       version;
 *   <li>is appended when the fragment's files cover none of its fields, as for a column added as
 *       all-NULL;
 *   <li>otherwise is appended after the fields it writes are tombstoned in the existing files that
 *       cover them. A file left with no live field is dropped. The fields may span several files,
 *       and fields no file covers yet need no tombstone, so one file can recompute a stored column
 *       and populate an all-NULL one: {@code frag: [A, B]} replaced with {@code [B, C]}, where
 *       {@code C} was added as all-NULL, becomes {@code frag: [A, -2] [B, C]}.
 * </ul>
 *
 * <p>Tombstoning a field of a legacy (v1) data file would misread its other fields, so a
 * replacement that needs one is refused. Indices on a replaced field no longer cover the replaced
 * fragments.
 */
public class DataReplacement implements Operation {
  private final List<DataReplacementGroup> replacements;

  private DataReplacement(List<DataReplacementGroup> replacements) {
    this.replacements = replacements;
  }

  /**
   * Get the list of data replacement groups.
   *
   * @return the list of data replacement groups
   */
  public List<DataReplacementGroup> replacements() {
    return replacements;
  }

  @Override
  public String name() {
    return "DataReplacement";
  }

  @Override
  public String toString() {
    return MoreObjects.toStringHelper(this).add("replacements", replacements).toString();
  }

  @Override
  public boolean equals(Object o) {
    if (this == o) return true;
    if (o == null || getClass() != o.getClass()) return false;
    DataReplacement that = (DataReplacement) o;
    return Objects.equals(replacements, that.replacements);
  }

  /**
   * Create a new builder for DataReplacement.
   *
   * @return a new builder
   */
  public static Builder builder() {
    return new Builder();
  }

  /** Builder for DataReplacement. */
  public static class Builder {
    private List<DataReplacementGroup> replacements;

    public Builder() {}

    /**
     * Set the list of data replacement groups.
     *
     * @param replacements the list of data replacement groups
     * @return this builder
     */
    public Builder replacements(List<DataReplacementGroup> replacements) {
      this.replacements = replacements;
      return this;
    }

    /**
     * Build a new DataReplacement.
     *
     * @return a new DataReplacement
     */
    public DataReplacement build() {
      return new DataReplacement(replacements);
    }
  }

  /** A group of data replacement, containing a fragment ID and a new data file. */
  public static class DataReplacementGroup {
    private final long fragmentId;
    private final DataFile replacedFile;

    /**
     * Create a new DataReplacementGroup.
     *
     * @param fragmentId the fragment ID
     * @param replacedFile the new data file to replace old file of the fragment id
     */
    public DataReplacementGroup(long fragmentId, DataFile replacedFile) {
      this.fragmentId = fragmentId;
      this.replacedFile = replacedFile;
    }

    /**
     * Get the fragment ID.
     *
     * @return the fragment ID
     */
    public long fragmentId() {
      return fragmentId;
    }

    /**
     * Get the new data file.
     *
     * @return the new data file
     */
    public DataFile replacedFile() {
      return replacedFile;
    }

    @Override
    public boolean equals(Object o) {
      if (this == o) return true;
      if (o == null || getClass() != o.getClass()) return false;
      DataReplacementGroup that = (DataReplacementGroup) o;
      return fragmentId == that.fragmentId && Objects.equals(replacedFile, that.replacedFile);
    }

    @Override
    public int hashCode() {
      return Objects.hash(fragmentId, replacedFile);
    }

    @Override
    public String toString() {
      return MoreObjects.toStringHelper(this)
          .add("fragmentId", fragmentId)
          .add("replacedFile", replacedFile)
          .toString();
    }
  }
}
